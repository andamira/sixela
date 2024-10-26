// sixela::tosixel

use alloc::{format, vec};
use devela::{Box, Vec, Write as IoWrite};

use crate::{
    dither::DitherConf,
    output::{sixel_node, sixel_output},
    pixelformat::sixel_helper_normalize_pixelformat,
    EncodePolicy, PaletteType, PixelFormat, SixelError, SixelResult, SIXEL_OUTPUT_PACKET_SIZE,
    SIXEL_PALETTE_MAX,
};

mod dither_fns;
use dither_fns::*;

const DCS_START_7BIT: &str = "\x1BP";
const DCS_START_8BIT: &str = "\u{220}";

const DCS_END_7BIT: &str = "\x1B\\";
const DCS_END_8BIT: &str = "\u{234}";
const SCREEN_PACKET_SIZE: usize = 256;

const PALETTE_HIT: i32 = 1;
const PALETTE_CHANGE: i32 = 2;
// enum Palette { // TODO
//     HIT,
//     CHANGE,
// }

impl<W: IoWrite> sixel_output<W> {
    /* GNU Screen penetration */
    fn penetrate(
        &mut self,
        nwrite: usize,   /* output size */
        dcs_start: &str, /* DCS introducer */
        dcs_end: &str,   /* DCS terminator */
    ) {
        let splitsize = SCREEN_PACKET_SIZE - dcs_start.len() - dcs_end.len();
        let mut pos = 0;
        while pos < nwrite {
            let _ = self.fn_write.write(dcs_start.as_bytes());
            let _ = self.fn_write.write(self.buffer[pos..pos + splitsize].as_bytes());
            let _ = self.fn_write.write(dcs_end.as_bytes());
            pos += splitsize;
        }
    }

    fn advance(&mut self) {
        if self.buffer.len() >= SIXEL_OUTPUT_PACKET_SIZE {
            if self.penetrate_multiplexer {
                self.penetrate(SIXEL_OUTPUT_PACKET_SIZE, DCS_START_7BIT, DCS_END_7BIT);
            } else {
                let _ = self.fn_write.write(self.buffer[..SIXEL_OUTPUT_PACKET_SIZE].as_bytes());
            }
            self.buffer.drain(0..SIXEL_OUTPUT_PACKET_SIZE);
        }
    }

    pub fn putc(&mut self, value: char) {
        self.buffer.push(value);
    }

    pub fn puts(&mut self, value: &str) {
        self.buffer.push_str(value);
    }

    pub(crate) fn puti(&mut self, i: i32) {
        self.puts(format!("{}", i).as_str());
    }

    pub(crate) fn putb(&mut self, b: u8) {
        self.puts(format!("{}", b).as_str());
    }

    pub fn put_flash(&mut self) -> SixelResult<()> {
        if self.has_gri_arg_limit {
            /* VT240 Max 255 ? */
            while self.save_count > 255 {
                /* argument of DECGRI('!') is limitted to 255 in real VT */
                self.puts("!255");
                self.advance();
                self.putc(unsafe { char::from_u32_unchecked(self.save_pixel as u32) });
                self.advance();
                self.save_count -= 255;
            }
        }
        if self.save_count > 3 {
            /* DECGRI Graphics Repeat Introducer ! Pn Ch */
            self.putc('!');
            self.advance();
            self.puti(self.save_count);
            self.advance();
            self.putc(unsafe { char::from_u32_unchecked(self.save_pixel as u32) });
            self.advance();
        } else {
            for _ in 0..self.save_count {
                self.putc(unsafe { char::from_u32_unchecked(self.save_pixel as u32) });
                self.advance();
            }
        }
        self.save_pixel = 0;
        self.save_count = 0;
        Ok(())
    }

    pub fn put_pixel(&mut self, mut pix: u8) -> SixelResult<()> {
        if pix > b'?' {
            pix = b'\0';
        }
        pix += b'?';
        if pix == self.save_pixel {
            self.save_count += 1;
        } else {
            self.put_flash()?;
            self.save_pixel = pix;
            self.save_count = 1;
        }
        Ok(())
    }

    pub fn put_node(
        &mut self,      /* output context */
        x: &mut i32,    /* header position */
        np: sixel_node, /* node object */
        ncolors: i32,   /* number of palette colors */
        keycolor: i32,
    ) -> SixelResult<()> {
        if ncolors != 2 || keycolor == -1 {
            /* designate palette index */
            if self.active_palette != np.pal {
                self.putc('#');
                self.advance();
                self.puti(np.pal);
                self.advance();
                self.active_palette = np.pal;
            }
        }

        while *x < np.sx {
            if *x != keycolor {
                self.put_pixel(0)?;
            }
            *x += 1;
        }
        while *x < np.mx {
            if *x != keycolor {
                self.put_pixel(np.map[*x as usize])?;
            }
            *x += 1;
        }
        self.put_flash()?;
        Ok(())
    }

    pub fn encode_header(&mut self, width: i32, height: i32) -> SixelResult<()> {
        let p = [0, 0, 0];
        let mut pcount = 3;

        let use_raster_attributes = true;

        if !self.skip_dcs_envelope {
            if self.has_8bit_control {
                self.puts(DCS_START_8BIT);
                self.advance();
            } else {
                self.puts(DCS_START_7BIT);
                self.advance();
            }
        }

        if p[2] == 0 {
            pcount -= 1;
            if p[1] == 0 {
                pcount -= 1;
                if p[0] == 0 {
                    pcount -= 1;
                }
            }
        }

        if pcount > 0 {
            self.puti(p[0]);
            self.advance();
            if pcount > 1 {
                self.putc(';');
                self.advance();
                self.puti(p[1]);
                self.advance();
                if pcount > 2 {
                    self.putc(';');
                    self.advance();
                    self.puti(p[2]);
                    self.advance();
                }
            }
        }

        self.putc('q');
        self.advance();

        if use_raster_attributes {
            self.puts("\"1;1;");
            self.advance();
            self.puti(width);
            self.advance();
            self.putc(';');
            self.advance();
            self.puti(height);
            self.advance();
        }

        Ok(())
    }

    pub fn output_rgb_palette_definition(
        &mut self,
        palette: &[u8],
        n: i32,
        keycolor: i32,
    ) -> SixelResult<()> {
        if n != keycolor {
            /* DECGCI Graphics Color Introducer  # Pc ; Pu; Px; Py; Pz */
            self.putc('#');
            self.advance();
            self.puti(n);
            self.advance();
            self.puts(";2;");
            self.advance();
            self.puti((palette[n as usize * 3] as i32 * 100 + 127) / 255);
            self.advance();
            self.putc(';');
            self.advance();
            self.puti((palette[n as usize * 3 + 1] as i32 * 100 + 127) / 255);
            self.advance();
            self.putc(';');
            self.advance();
            self.puti((palette[n as usize * 3 + 2] as i32 * 100 + 127) / 255);
            self.advance();
        }
        Ok(())
    }

    pub fn output_hls_palette_definition(
        &mut self,
        palette: &[u8],
        n: i32,
        keycolor: i32,
    ) -> SixelResult<()> {
        if n != keycolor {
            let n = n as usize;
            let r = palette[n * 3 + 0] as i32;
            let g = palette[n * 3 + 1] as i32;
            let b = palette[n * 3 + 2] as i32;
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let l = ((max + min) * 100 + 255) / 510;
            let mut h = 0;
            let mut s = 0;

            if max == min {
                // h = s = 0;
            } else {
                if l < 50 {
                    s = ((max - min) * 100) / (max + min);
                } else {
                    s = ((max - min) * 100) / ((255 - max) + (255 - min));
                }
                if r == max {
                    h = 120 + (g - b) * 60 / (max - min);
                } else if g == max {
                    h = 240 + (b - r) * 60 / (max - min);
                } else if r < g
                /* if b == max */
                {
                    h = 360 + (r - g) * 60 / (max - min);
                } else {
                    h = 0 + (r - g) * 60 / (max - min);
                }
            }
            /* DECGCI Graphics Color Introducer  # Pc ; Pu; Px; Py; Pz */
            self.putc('#');
            self.advance();
            self.puti(n as i32);
            self.advance();
            self.puts(";1;");
            self.advance();
            self.puti(h);
            self.advance();
            self.putc(';');
            self.advance();
            self.puti(l);
            self.advance();
            self.putc(';');
            self.advance();
            self.puti(s);
            self.advance();
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn encode_body(
        &mut self,
        pixels: &[u8],
        width: i32,
        height: i32,
        palette: &[u8],
        ncolors: usize,
        keycolor: i32,
        bodyonly: bool,
        palstate: Option<&[i32]>,
    ) -> SixelResult<()> {
        if palette.is_empty() {
            return Err(Box::new(SixelError::BadArgument));
        }
        let len = ncolors * width as usize;
        self.active_palette = -1;

        let mut map: Vec<u8> = vec![0; len];

        if !bodyonly && (ncolors != 2 || keycolor == (-1)) {
            if matches!(self.palette_type, PaletteType::HLS) {
                for n in 0..ncolors {
                    self.output_hls_palette_definition(palette, n as i32, keycolor)?;
                }
            } else {
                for n in 0..ncolors {
                    self.output_rgb_palette_definition(palette, n as i32, keycolor)?;
                }
            }
        }
        let mut i = 0;
        let mut fillable: bool;
        let mut pix;

        for y in 0..height {
            if self.encode_policy != EncodePolicy::Size {
                fillable = false;
            } else if palstate.is_some() {
                /* high color sixel */
                pix = pixels[((y - i) * width) as usize] as i32;
                fillable = pix as usize >= ncolors;
            } else {
                /* normal sixel */
                fillable = true;
            }
            for x in 0..width {
                if y > i32::MAX / width {
                    /* integer overflow */
                    /*sixel_helper_set_additional_message(
                    "sixel_encode_body: integer overflow detected."
                    " (y > INT_MAX)");*/
                    return Err(Box::new(SixelError::BadIntegerOverflow));
                }
                let mut check_integer_overflow = y * width;
                if check_integer_overflow > i32::MAX - x {
                    /* integer overflow */
                    /*sixel_helper_set_additional_message(
                    "sixel_encode_body: integer overflow detected."
                    " (y * width > INT_MAX - x)");*/
                    return Err(Box::new(SixelError::BadIntegerOverflow));
                }
                pix = pixels[(check_integer_overflow + x) as usize] as i32; /* color index */
                if pix >= 0 && (pix as usize) < ncolors && pix != keycolor {
                    if pix > i32::MAX / width {
                        /* integer overflow */
                        /*sixel_helper_set_additional_message(
                        "sixel_encode_body: integer overflow detected."
                        " (pix > INT_MAX / width)");*/
                        return Err(Box::new(SixelError::BadIntegerOverflow));
                    }
                    check_integer_overflow = pix * width;
                    if check_integer_overflow > i32::MAX - x {
                        /* integer overflow */
                        /*sixel_helper_set_additional_message(
                        "sixel_encode_body: integer overflow detected."
                        " (pix * width > INT_MAX - x)");*/
                        return Err(Box::new(SixelError::BadIntegerOverflow));
                    }
                    map[(pix * width + x) as usize] |= 1 << i;
                } else if palstate.is_none() {
                    fillable = false;
                }
            }

            i += 1;
            if i < 6 && (y + 1) < height {
                continue;
            }
            for c in 0..ncolors {
                let mut sx = 0;
                while sx < width {
                    if map[c * width as usize + sx as usize] == 0 {
                        sx += 1;
                        continue;
                    }
                    let mut mx = sx + 1;
                    while mx < width {
                        if map[c * width as usize + mx as usize] != 0 {
                            mx += 1;
                            continue;
                        }
                        let mut n = 1;
                        while (mx + n) < width {
                            if map[c * width as usize + mx as usize + n as usize] != 0 {
                                break;
                            }
                            n += 1;
                        }

                        if n >= 10 || (mx + n) >= width {
                            break;
                        }
                        mx = mx + n - 1;
                        mx += 1;
                    }
                    let np = sixel_node {
                        pal: c as i32,
                        sx,
                        mx,
                        map: map[c * width as usize..].to_vec(),
                    };

                    self.nodes.insert(0, np);
                    sx = mx - 1;
                    sx += 1;
                }
            }

            if y != 5 {
                /* DECGNL Graphics Next Line */
                self.putc('-');
                self.advance();
            }
            let mut x = 0;
            while let Some(mut np) = self.nodes.pop() {
                if x > np.sx {
                    /* DECGCR Graphics Carriage Return */
                    self.putc('$');
                    self.advance();
                    x = 0;
                }

                if fillable {
                    // memset(np->map + np->sx, (1 << i) - 1, (size_t)(np->mx - np->sx));
                    let v = (1 << i) - 1;
                    np.map.resize(np.mx as usize, v);
                    for j in np.sx..np.mx {
                        np.map[j as usize] = v;
                    }
                }
                self.put_node(&mut x, np, ncolors as i32, keycolor)?;

                let mut ni = self.nodes.len() as i32 - 1;
                while ni >= 0 {
                    let onode = &self.nodes[ni as usize];

                    if onode.sx < x {
                        ni -= 1;
                        continue;
                    }

                    if fillable {
                        // memset(np.map + np.sx, (1 << i) - 1, (size_t)(np.mx - np.sx));
                        let np = &mut self.nodes[ni as usize];
                        let v = (1 << i) - 1;
                        np.map.resize(np.mx as usize, v);
                        for j in np.sx..np.mx {
                            np.map[j as usize] = v;
                        }
                    }
                    let np = self.nodes.remove(ni as usize);
                    self.put_node(&mut x, np, ncolors as i32, keycolor)?;
                    ni -= 1;
                }

                fillable = false;
            }

            i = 0;
            map.clear();
            map.resize(len, 0);
        }

        if palstate.is_some() {
            self.putc('$');
            self.advance();
        }
        Ok(())
    }

    pub fn encode_footer(&mut self) -> SixelResult<()> {
        if !self.skip_dcs_envelope && !self.penetrate_multiplexer {
            if self.has_8bit_control {
                self.puts(DCS_END_8BIT);
                self.advance();
            } else {
                self.puts(DCS_END_7BIT);
                self.advance();
            }
        }

        /* flush buffer */
        if !self.buffer.is_empty() {
            if self.penetrate_multiplexer {
                self.penetrate(self.buffer.len(), DCS_START_7BIT, DCS_END_7BIT);
                let _ = self.fn_write.write(b"\x1B\\");
            } else {
                let _ = self.fn_write.write(self.buffer.as_bytes());
            }
        }
        Ok(())
    }

    pub fn encode_dither(
        &mut self,
        pixels: &[u8],
        width: i32,
        height: i32,
        dither: &mut DitherConf,
    ) -> SixelResult<()> {
        let input_pixels = match dither.pixelformat {
            PixelFormat::PAL1
            | PixelFormat::PAL2
            | PixelFormat::PAL4
            | PixelFormat::G1
            | PixelFormat::G2
            | PixelFormat::G4 => {
                let mut paletted_pixels = vec![0; (width * height * 3) as usize];
                dither.pixelformat = sixel_helper_normalize_pixelformat(
                    &mut paletted_pixels,
                    pixels,
                    dither.pixelformat,
                    width,
                    height,
                )?;
                paletted_pixels
            }

            PixelFormat::PAL8 | PixelFormat::G8 | PixelFormat::GA88 | PixelFormat::AG88 => {
                pixels.to_vec()
            }

            _ => {
                /* apply palette */
                dither.apply_palette(pixels, width, height)?
            }
        };
        self.encode_header(width, height)?;
        self.encode_body(
            &input_pixels,
            width,
            height,
            &dither.palette,
            dither.ncolors as usize,
            dither.keycolor,
            dither.bodyonly,
            None,
        )?;
        self.encode_footer()?;
        Ok(())
    }

    pub fn encode_highcolor(
        &mut self,
        pixels: &mut [u8],
        width: i32,
        mut height: i32,
        dither: &mut DitherConf,
    ) -> SixelResult<()> {
        let maxcolors = 1 << 15;
        let mut px_idx = 0;
        let mut normalized_pixels = vec![0; (width * height * 3) as usize];
        let pixels = if !matches!(dither.pixelformat, PixelFormat::BGR888) {
            /* normalize pixelfromat */
            sixel_helper_normalize_pixelformat(
                &mut normalized_pixels,
                pixels,
                dither.pixelformat,
                width,
                height,
            )?;
            &mut normalized_pixels
        } else {
            pixels
        };
        let mut paletted_pixels: Vec<u8> = vec![0; (width * height) as usize];
        let mut rgbhit = vec![0; maxcolors as usize];
        let mut rgb2pal = vec![0; maxcolors as usize];
        // let marks = &mut rgb2pal[maxcolors as usize..];
        let mut output_count = 0;

        let mut is_running = true;
        let mut palstate: Vec<i32> = vec![0; SIXEL_PALETTE_MAX];
        let mut palhitcount: Vec<i32> = vec![0; SIXEL_PALETTE_MAX];
        let mut marks = vec![false; (width * 6) as usize];
        while is_running {
            let mut dst = 0;
            let mut nextpal: usize = 0;
            let mut threshold = 1;
            let mut dirty = false;
            let mut mptr = 0;
            marks.clear();
            marks.resize((width * 6) as usize, false);
            palstate.clear();
            palstate.resize(SIXEL_PALETTE_MAX, 0);
            let mut y = 0;
            let mut mod_y = 0;

            loop {
                for x in 0..width {
                    if marks[mptr] {
                        paletted_pixels[dst] = 255;
                    } else {
                        sixel_apply_15bpp_dither(
                            &mut pixels[px_idx..],
                            x,
                            y,
                            width,
                            height,
                            dither.method_for_diffuse,
                        );
                        let pix = ((pixels[px_idx] & 0xf8) as i32) << 7
                            | ((pixels[px_idx + 1] & 0xf8) as i32) << 2
                            | ((pixels[px_idx + 2] >> 3) & 0x1f) as i32;

                        if rgbhit[pix as usize] == 0 {
                            loop {
                                if nextpal >= 255 {
                                    if threshold >= 255 {
                                        break;
                                    } else {
                                        threshold = if threshold == 1 { 9 } else { 255 };
                                        nextpal = 0;
                                    }
                                } else if palstate[nextpal] != 0 || palhitcount[nextpal] > threshold
                                {
                                    nextpal += 1;
                                } else {
                                    break;
                                }
                            }

                            if nextpal >= 255 {
                                dirty = true;
                                paletted_pixels[dst] = 255;
                            } else {
                                let pal = nextpal * 3;

                                rgbhit[pix as usize] = 1;
                                if output_count > 0 {
                                    rgbhit[((dither.palette[pal] as usize & 0xf8) << 7)
                                        | ((dither.palette[pal + 1] as usize & 0xf8) << 2)
                                        | ((dither.palette[pal + 2] as usize >> 3) & 0x1f)] = 0;
                                }
                                paletted_pixels[dst] = nextpal as u8;
                                rgb2pal[pix as usize] = nextpal as u8;
                                nextpal += 1;
                                marks[mptr] = true;
                                palstate[paletted_pixels[dst] as usize] = PALETTE_CHANGE;
                                palhitcount[paletted_pixels[dst] as usize] = 1;
                                dither.palette[pal] = pixels[px_idx + 0];
                                dither.palette[pal + 1] = pixels[px_idx + 1];
                                dither.palette[pal + 2] = pixels[px_idx + 2];
                            }
                        } else {
                            let pp = rgb2pal[pix as usize];
                            paletted_pixels[dst] = pp;
                            let pp = pp as usize;

                            marks[mptr] = true;
                            if palstate[pp] != 0 {
                                palstate[pp] = PALETTE_HIT;
                            }
                            if palhitcount[pp] < 255 {
                                palhitcount[pp] += 1;
                            }
                        }
                    }

                    mptr += 1;
                    dst += 1;
                    px_idx += 3;
                }
                y += 1;
                if y >= height {
                    if dirty {
                        mod_y = 5;
                    } else {
                        is_running = false;
                        break;
                    }
                }
                if dirty && (mod_y == 5 || y >= height) {
                    let orig_height = height;

                    if output_count == 0 {
                        self.encode_header(width, height)?;
                    }
                    output_count += 1;

                    height = y;

                    self.encode_body(
                        &paletted_pixels,
                        width,
                        height,
                        &dither.palette,
                        dither.ncolors as usize,
                        255,
                        dither.bodyonly,
                        Some(&palstate),
                    )?;
                    if y >= orig_height {
                        // end outer loop
                        is_running = false;
                        break;
                    }
                    px_idx -= (6 * width * 3) as usize;
                    height = orig_height - height + 6;
                    break; // goto next outer loop
                }
                mod_y += 1;
                if mod_y == 6 {
                    marks.clear();
                    marks.resize(maxcolors as usize, false);
                    mptr = 0;
                    mod_y = 0;
                }
            }
        }
        if output_count == 0 {
            self.encode_header(width, height)?;
        }

        let _ = self.encode_body(
            &paletted_pixels,
            width,
            height,
            &dither.palette,
            dither.ncolors as usize,
            255,
            dither.bodyonly,
            Some(&palstate),
        );

        let _ = self.encode_footer();

        Ok(())
    }

    pub fn encode(
        &mut self,
        pixels: &mut [u8],
        width: i32,
        height: i32,
        _depth: i32, /* color depth */
        dither: &mut DitherConf,
    ) -> SixelResult<()> /* output context */ {
        /*
            println!("sixel_encode: {} x {} depth {}", width, height, _depth);
            println!("dither:");
            println!("\treqcolors: {}", dither.reqcolors);
            println!("\tncolors: {}", dither.ncolors);
            println!("\torigcolors: {}", dither.origcolors);
            println!("\toptimized: {}", dither.optimized);
            println!("\toptimize_palette: {}", dither.optimize_palette);
            println!("\tcomplexion: {}", dither.complexion);
            println!("\tbodyonly: {}", dither.bodyonly);
            println!("\tmethod_for_largest: {:?}", dither.method_for_largest as i32);
            println!("\tmethod_for_rep: {:?}", dither.method_for_rep as i32);
            println!("\tmethod_for_diffuse: {:?}", dither.method_for_diffuse as i32);
            println!("\tquality_mode: {:?}", dither.quality_mode as i32);
            println!("\tkeycolor: {:?}", dither.keycolor);
            println!("\tpixelformat: {:?}", dither.pixelformat as i32);
        */
        if width < 1 {
            return Err(Box::new(SixelError::BadInput));
            /*
            sixel_helper_set_additional_message(
                "sixel_encode: bad width parameter."
                " (width < 1)");
            status = SIXEL_BAD_INPUT;
            goto end;*/
        }

        if height < 1 {
            return Err(Box::new(SixelError::BadInput));
            /*
            sixel_helper_set_additional_message(
                "sixel_encode: bad height parameter."
                " (height < 1)");
            status = SIXEL_BAD_INPUT;
            goto end;*/
        }
        match dither.quality_mode {
            crate::Quality::Auto
            | crate::Quality::High
            | crate::Quality::Low
            | crate::Quality::Full => {
                self.encode_dither(pixels, width, height, dither)?;
            }
            crate::Quality::HighColor => {
                self.encode_highcolor(pixels, width, height, dither)?;
            }
        }
        Ok(())
    }
}
