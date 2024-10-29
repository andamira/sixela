// sixela::output::builder
//

use crate::{
    DitherConf, EncodePolicy, LargestDim, PixelFormat, Quality, RepColor, SixelDiffusion,
    SixelError, SixelOutput, SixelResult,
};
use devela::{ConstDefault, String, ToString, Vec};

/// A configurable sixel builder from a slice of pixel data.
///
/// By default it assumes `RGB888` PixelFormat, and `Auto`matic `SixelDiffusion`,
/// `LargestDim`, `RepColor` and `Quality`.
///
/// # Example
/// ```
/// # use sixela::Sixel;
/// // 2x2 pixels (Red, Green, Blue, White)
/// const IMAGE_HEX: &[u8] = b"FF000000FF000000FFFFFFFF";
/// //                         RRGGBBrrggbbRRGGBBrrggbb
/// println!("{}", Sixel::with_bytes_size(IMAGE_HEX, 2, 2).build().unwrap());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Sixel<'a> {
    pub bytes: Option<&'a [u8]>,
    pub width: i32,
    pub height: i32,
    pub format: PixelFormat,
    pub diffuse: SixelDiffusion,
    pub largest: LargestDim,
    pub rep: RepColor,
    pub quality: Quality,
}
impl<'a> ConstDefault for Sixel<'a> {
    const DEFAULT: Self = Self {
        bytes: None,
        width: 0,
        height: 0,
        format: PixelFormat::DEFAULT,
        diffuse: SixelDiffusion::DEFAULT,
        largest: LargestDim::DEFAULT,
        rep: RepColor::DEFAULT,
        quality: Quality::DEFAULT,
    };
}

/// # Common methods
#[rustfmt::skip]
impl<'a> Sixel<'a> {
    /// Returns a new empty sixel builder.
    #[inline] #[must_use]
    pub const fn new() -> Self { Self::DEFAULT }

    /// Returns a new empty sixel builder with the given byte slice.
    #[inline] #[must_use]
    pub const fn with_bytes(bytes: &'a [u8]) -> Self {
        Self::DEFAULT.bytes(bytes)
    }

    /// Returns a new empty sixel builder with the given size.
    #[inline] #[must_use]
    pub const fn with_size(width: i32, height: i32) -> Self {
        Self::DEFAULT.size(width, height)
    }

    /// Returns a new empty sixel builder with the given byte slize and size.
    #[inline] #[must_use]
    pub const fn with_bytes_size(bytes: &'a [u8], width: i32, height: i32) -> Self {
        Self::DEFAULT.bytes(bytes).size(width, height)
    }

    /* */

    /// Builds a sixel formatted string with the configured options.
    ///
    /// # Errors
    /// Returns an error if the bytes slice have not been set,
    /// if either the width or height is 0,
    /// or the slice is not long enough.
    pub fn build(self) -> SixelResult<String> {
        if self.width == 0 || self.height == 0 {
            return Err(SixelError::BadInput);
        }
        if let Some(bytes) = self.bytes {
            if bytes.len() < self.format.required_bytes(self.width, self.height) {
                Err(SixelError::BadInput)
            } else {
                sixel_string(bytes, self.width, self.height,
                    self.format, self.diffuse, self.largest, self.rep, self.quality)
            }
        } else {
            Err(SixelError::BadInput)
        }
    }

    /* */

    /// Sets the byte slice of image data.
    #[inline] #[must_use]
    pub const fn bytes(mut self, bytes: &'a [u8]) -> Self {
        self.bytes = Some(bytes); self
    }
    /// Sets the width.
    #[inline] #[must_use]
    pub const fn width(mut self, width: i32) -> Self {
        self.width = width; self
    }
    /// Sets the height.
    #[inline] #[must_use]
    pub const fn height(mut self, height: i32) -> Self {
        self.height = height; self
    }
    /// Sets the size (width, height).
    #[inline] #[must_use]
    pub const fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /**/

    /// Sets the pixel format.
    #[inline] #[must_use]
    pub const fn format(mut self, format: PixelFormat) -> Self {
        self.format = format; self
    }
    /// Sets the method for diffusion.
    #[inline] #[must_use]
    pub const fn diffuse(mut self, diffuse: SixelDiffusion) -> Self {
        self.diffuse = diffuse; self
    }
    /// Sets the method for largest.
    #[inline] #[must_use]
    pub const fn largest(mut self, largest: LargestDim) -> Self {
        self.largest = largest; self
    }
    /// Sets the method for rep.
    #[inline] #[must_use]
    pub const fn rep(mut self, rep: RepColor) -> Self {
        self.rep = rep; self
    }
    /// Sets the quality.
    #[inline] #[must_use]
    pub const fn quality(mut self, quality: Quality) -> Self {
        self.quality = quality; self
    }
}

macro_rules! add_method {
    ($fn:ident, $field:ident, $variant:expr) => {
        #[doc = concat!["Sets the `", stringify!($field), "` field to [`", stringify!($variant), "`]."]]
        #[inline]
        #[must_use]
        pub const fn $fn(mut self) -> Self {
            self.$field = $variant;
            self
        }
    };
}

/// # Extra methods
#[rustfmt::skip]
impl<'a> Sixel<'a> {
    add_method![format_rgb555, format, PixelFormat::RGB555];
    add_method![format_rgb565, format, PixelFormat::RGB565];
    add_method![format_rgb888, format, PixelFormat::RGB888];
    add_method![format_bgr555, format, PixelFormat::BGR555];

    add_method![format_bgr565, format, PixelFormat::BGR565];
    add_method![format_bgr888, format, PixelFormat::BGR888];
    add_method![format_argb8888, format, PixelFormat::ARGB8888];
    add_method![format_rgba8888, format, PixelFormat::RGBA8888];
    add_method![format_abgr8888, format, PixelFormat::ABGR8888];
    add_method![format_bgra8888, format, PixelFormat::BGRA8888];
    add_method![format_g1, format, PixelFormat::G1];
    add_method![format_g2, format, PixelFormat::G2];
    add_method![format_g4, format, PixelFormat::G4];
    add_method![format_g8, format, PixelFormat::G8];
    add_method![format_ag88, format, PixelFormat::AG88];
    add_method![format_ga88, format, PixelFormat::GA88];
    add_method![format_pal1, format, PixelFormat::PAL1];
    add_method![format_pal2, format, PixelFormat::PAL2];
    add_method![format_pal4, format, PixelFormat::PAL4];
    add_method![format_pal8, format, PixelFormat::PAL8];
    //
    add_method![largest_auto, largest, LargestDim::Auto];
    add_method![largest_norm, largest, LargestDim::Norm];
    add_method![largest_lum, largest, LargestDim::Lum];
    //
    add_method![rep_auto, rep, RepColor::Auto];
    add_method![rep_center, rep, RepColor::Center];
    add_method![rep_average, rep, RepColor::AverageColors];
    add_method![rep_pixels, rep, RepColor::AveragePixels];
    //
    add_method![diffuse_auto, diffuse, SixelDiffusion::Auto];
    add_method![diffuse_none, diffuse, SixelDiffusion::None];
    add_method![diffuse_atkinson, diffuse, SixelDiffusion::Atkinson];
    add_method![diffuse_fs, diffuse, SixelDiffusion::FS];
    add_method![diffuse_jajuni, diffuse, SixelDiffusion::JaJuNi];
    add_method![diffuse_stucki, diffuse, SixelDiffusion::Stucki];
    add_method![diffuse_burkes, diffuse, SixelDiffusion::Burkes];
    add_method![diffuse_adither, diffuse, SixelDiffusion::ADither];
    add_method![diffuse_xdither, diffuse, SixelDiffusion::XDither];
    //
    add_method![quality_auto, quality, Quality::Auto];
    add_method![quality_high, quality, Quality::High];
    add_method![quality_low, quality, Quality::Low];
    add_method![quality_full, quality, Quality::Full];
    add_method![quality_high_color, quality, Quality::HighColor];
}

/// Writes a string of sixel data.
///
/// # Example
/// ```ignore
/// # use sixela::*;
/// // 2x2 pixels (Red, Green, Blue, White)
/// const IMAGE_HEX: &[u8] = b"FF000000FF000000FFFFFFFF";
///                          //RRGGBBrrggbbRRGGBBrrggbb
///
/// println!("{}", sixel_string(
///     IMAGE_HEX, 2, 2,
///     PixelFormat::RGB888,
///     SixelDiffusion::Stucki,
///     LargestDim::Auto,
///     RepColor::Auto,
///     Quality::Auto
/// ).unwrap());
/// ```
#[expect(clippy::too_many_arguments)]
fn sixel_string(
    bytes: &[u8],
    width: i32,
    height: i32,
    pixelformat: PixelFormat,
    method_for_diffuse: SixelDiffusion,
    method_for_largest: LargestDim,
    method_for_rep: RepColor,
    quality_mode: Quality,
) -> SixelResult<String> {
    let mut sixel_data: Vec<u8> = Vec::new(); // MAYBE with_capacity

    let mut sixel_output = SixelOutput::new(&mut sixel_data);
    sixel_output.set_encode_policy(EncodePolicy::Auto);
    let mut dither_conf = DitherConf::new(256).unwrap();

    dither_conf.set_optimize_palette(true);

    dither_conf.initialize(
        bytes,
        width,
        height,
        pixelformat,
        method_for_largest,
        method_for_rep,
        quality_mode,
    )?;
    dither_conf.set_pixelformat(pixelformat);
    dither_conf.set_diffusion_method(method_for_diffuse);

    let mut bytes = bytes.to_vec();
    sixel_output.encode(&mut bytes, width, height, 0, &mut dither_conf)?;

    Ok(String::from_utf8_lossy(&sixel_data).to_string())
}
