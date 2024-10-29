// sixela::dither

mod palettes;
use palettes::*;

use alloc::vec;
use devela::Vec;

use crate::{
    pixelformat::sixel_helper_normalize_pixelformat,
    quant::{sixel_quant_apply_palette, sixel_quant_make_palette},
    MethodForLargest, MethodForRep, PixelFormat, Quality, SixelDiffusion, SixelError, SixelResult,
    SIXEL_PALETTE_MAX,
};

/// Predefined dithering modes for sixel output.
///
/// # Adaptation
/// - Derived from `sixel_builtin_dither_t` in the `libsixel` C library.
/// - Represents various terminal and grayscale dithering options.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DitherMode {
    /// monochrome terminal with dark background
    MonoDark,
    /// monochrome terminal with light background
    MonoLight,
    /// xterm 16color
    XTerm16,
    /// xterm 256color
    XTerm256,
    /// vt340 monochrome
    VT340Mono,
    /// vt340 color
    VT340Color,
    /// 1bit grayscale
    G1,
    /// 2bit grayscale
    G2,
    /// 4bit grayscale
    G4,
    /// 8bit grayscale
    G8,
}

/// Configuration for sixel dithering.
///
///
/// # Adaptation
/// - Based on `sixel_dither_t` from the `libsixel` C library,
///   adapted with adjustments for idiomatic Rust usage.
#[derive(Debug)]
pub struct DitherConf {
    /// Palette definition.
    pub palette: Vec<u8>,
    /// Cache table.
    pub cachetable: Option<Vec<u16>>,
    /// The number of requested colors.
    pub reqcolors: i32,
    /// The number of active colors.
    pub ncolors: i32,
    /// The number of original colors.
    pub origcolors: i32,
    /// Pixel is 15bpp compressible.
    pub optimized: bool,
    /// Minimize palette size.
    pub optimize_palette: bool,
    /// For complexion correction.
    pub complexion: i32,
    /// Do not output palette section if true.
    pub bodyonly: bool,
    /// Method for finding the largest dimention for splitting.
    pub method_for_largest: MethodForLargest,
    /// Method for choosing a color from the box.
    pub method_for_rep: MethodForRep,
    /// Method for diffusing
    pub method_for_diffuse: SixelDiffusion,
    /// Quality of histogram.
    pub quality_mode: Quality,
    /// Background color.
    pub keycolor: i32,
    /// Pixelformat for internal processing.
    pub pixelformat: PixelFormat,
}

impl DitherConf {
    /// Creates a new dither configuration with the specified number of colors.
    pub fn new(mut ncolors: i32) -> SixelResult<Self> {
        let quality_mode = if ncolors < 0 {
            ncolors = SIXEL_PALETTE_MAX as i32;
            Quality::HighColor
        } else {
            if ncolors > SIXEL_PALETTE_MAX as i32 {
                return Err(SixelError::BadInput);
            }
            if ncolors < 1 {
                return Err(SixelError::BadInput);
                // sixel_helper_set_additional_message(
                // "DitherConf::new: palette ncolors must be more than 0");
            }
            Quality::Low
        };
        Ok(Self {
            palette: vec![0; ncolors as usize * 3],
            cachetable: None,
            reqcolors: ncolors,
            ncolors,
            origcolors: (-1),
            keycolor: (-1),
            optimized: false,
            optimize_palette: false,
            complexion: 1,
            bodyonly: false,
            method_for_largest: MethodForLargest::Norm,
            method_for_rep: MethodForRep::CenterBox,
            method_for_diffuse: SixelDiffusion::FS,
            quality_mode,
            pixelformat: PixelFormat::RGB888,
        })
    }

    /// TODO
    pub fn new_mode(dither_mode: DitherMode) -> SixelResult<Self> {
        let (ncolors, palette, keycolor) = match dither_mode {
            DitherMode::MonoDark => (2, pal_mono_dark.to_vec(), 0),
            DitherMode::MonoLight => (2, pal_mono_light.to_vec(), 0),
            DitherMode::XTerm16 => (16, pal_xterm256.to_vec(), -1),
            DitherMode::XTerm256 => (256, pal_xterm256.to_vec(), -1),
            DitherMode::VT340Mono => (16, pal_vt340_mono.to_vec(), -1),
            DitherMode::VT340Color => (16, pal_vt340_color.to_vec(), -1),
            DitherMode::G1 => (2, pal_gray_1bit.to_vec(), -1),
            DitherMode::G2 => (4, pal_gray_2bit.to_vec(), -1),
            DitherMode::G4 => (16, pal_gray_4bit.to_vec(), -1),
            DitherMode::G8 => (256, pal_gray_8bit.to_vec(), -1),
        };

        let mut result = DitherConf::new(ncolors)?;
        result.palette = palette;
        result.keycolor = keycolor;
        result.optimized = true;
        result.optimize_palette = false;
        Ok(result)
    }

    /// TODO
    pub fn set_method_for_largest(&mut self, method_for_largest: MethodForLargest) {
        self.method_for_largest = if matches!(method_for_largest, MethodForLargest::Auto) {
            MethodForLargest::Norm
        } else {
            method_for_largest
        };
    }

    /// TODO
    pub fn set_method_for_rep(&mut self, method_for_rep: MethodForRep) {
        self.method_for_rep = if matches!(method_for_rep, MethodForRep::Auto) {
            MethodForRep::CenterBox
        } else {
            method_for_rep
        };
    }

    /// TODO
    pub fn set_quality_mode(&mut self, quality_mode: Quality) {
        self.quality_mode = if matches!(quality_mode, Quality::Auto) {
            if self.ncolors <= 8 {
                Quality::High
            } else {
                Quality::Low
            }
        } else {
            quality_mode
        };
    }

    /// TODO
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        data: &[u8],
        width: i32,
        height: i32,
        pixelformat: PixelFormat,
        method_for_largest: MethodForLargest,
        method_for_rep: MethodForRep,
        quality_mode: Quality,
    ) -> SixelResult<()> {
        self.set_pixelformat(pixelformat);
        #[expect(clippy::single_match_else, reason = "could be extended")]
        let input_pixels = match pixelformat {
            PixelFormat::RGB888 => data.to_vec(),
            _ => {
                /* normalize pixelformat */
                let mut normalized_pixels = vec![0; (width * height * 3) as usize];
                self.set_pixelformat(sixel_helper_normalize_pixelformat(
                    &mut normalized_pixels,
                    data,
                    pixelformat,
                    width,
                    height,
                )?);
                normalized_pixels
            }
        };

        self.set_method_for_largest(method_for_largest);
        self.set_method_for_rep(method_for_rep);
        self.set_quality_mode(quality_mode);

        let buf = sixel_quant_make_palette(
            &input_pixels,
            width * height * 3,
            PixelFormat::RGB888,
            self.reqcolors,
            &mut self.ncolors,
            &mut self.origcolors,
            self.method_for_largest,
            self.method_for_rep,
            self.quality_mode,
        )?;

        self.palette = buf;
        self.optimized = true;
        if self.origcolors <= self.reqcolors {
            self.method_for_diffuse = SixelDiffusion::None;
        }
        Ok(())
    }

    /// Set diffusion method.
    pub fn set_diffusion_method(&mut self, method: SixelDiffusion) {
        self.method_for_diffuse = if matches!(method, SixelDiffusion::Auto) {
            if self.ncolors > 16 {
                SixelDiffusion::FS
            } else {
                SixelDiffusion::Atkinson
            }
        } else {
            method
        };
    }

    /// Get number of palette colors.
    #[inline]
    pub fn get_num_of_palette_colors(&self) -> i32 {
        self.ncolors
    }

    /// Get number of histogram colors.
    #[inline]
    pub fn get_num_of_histogram_colors(&self) -> i32 {
        self.origcolors
    }

    /// Get the palette.
    #[inline]
    pub fn get_palette(&self) -> &[u8] {
        &self.palette
    }

    /// Set the palette.
    #[inline]
    pub fn set_palette(&mut self, palette: Vec<u8>) {
        self.palette = palette;
    }

    /// set the factor of complexion color correcting
    //  /* complexion score (>= 1) */
    #[inline]
    pub fn set_complexion_score(&mut self, score: i32) {
        self.complexion = score;
    }

    /// Set whether omitting palette definition.
    ///
    /// false: outputs palette section.
    #[inline]
    pub fn set_body_only(&mut self, bodyonly: bool) {
        self.bodyonly = bodyonly;
    }

    /// Set whether optimize palette size.
    ///
    /// false: optimizes the palette size.
    #[inline]
    pub fn set_optimize_palette(&mut self, do_op: bool) {
        self.optimize_palette = do_op;
    }

    /// Set the pixel format
    #[inline]
    pub fn set_pixelformat(&mut self, pixelformat: PixelFormat) {
        self.pixelformat = pixelformat;
    }

    /// Set the transparent color index.
    #[inline]
    pub fn set_transparent(&mut self, index: i32) {
        self.keycolor = index;
    }

    /* set transparent */
    pub fn apply_palette(
        &mut self,
        pixels: &[u8],
        width: i32,
        height: i32,
    ) -> SixelResult<Vec<u8>> {
        let bufsize = width * height;
        let mut dest = vec![0; bufsize as usize];

        /* if quality_mode is full, do not use palette caching */
        if matches!(self.quality_mode, Quality::Full) {
            self.optimized = false;
        }

        if self.cachetable.is_none()
            && self.optimized
            && self.palette != pal_mono_dark
            && self.palette != pal_mono_light
        {
            self.cachetable = Some(vec![0; 1 << (3 * 5)]);
        }

        let mut input_pixels = if !matches!(self.pixelformat, PixelFormat::RGB888) {
            /* normalize pixelformat */
            let mut normalized_pixels = vec![0; (width * height * 3) as usize];
            self.pixelformat = sixel_helper_normalize_pixelformat(
                &mut normalized_pixels,
                pixels,
                self.pixelformat,
                width,
                height,
            )?;
            normalized_pixels
        } else {
            pixels.to_vec()
        };
        let ncolors = sixel_quant_apply_palette(
            &mut dest,
            &mut input_pixels,
            width,
            height,
            3,
            &mut self.palette,
            self.ncolors,
            self.method_for_diffuse,
            self.optimized,
            self.optimize_palette,
            self.complexion,
            Some(self.cachetable.as_mut().unwrap()),
        )?;
        self.ncolors = ncolors;

        Ok(dest)
    }
}
