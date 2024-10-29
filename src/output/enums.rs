// sixela::output::enums
//
// TOC
// - enum LargestDim
// - enum RepColor
// - enum SixelDiffusion
// - enum Quality
// - enum PixelFormat
// - enum EncodePolicy
// - enum PaletteType
// - enum Loop
// - //
//   - enum ResampleMethod
//   - enum Format
//   - enum FormatType

use devela::ConstDefault;

/// Method for finding the largest dimension for splitting,
/// and sorting by that component.
///
/// # Adaptation
/// - Derived from `methodForLargest` enum in the `libsixel` C library.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum LargestDim {
    /// Choose automatically the method for finding the largest dimension. (default)
    #[default]
    Auto,
    /// Simply comparing the range in RGB space.
    Norm,
    /// Transforming into luminosities before the comparison.
    Lum,
}
#[rustfmt::skip]
impl ConstDefault for LargestDim { const DEFAULT: Self = Self::Auto; }

/// Method for choosing a representative color from the box.
///
/// # Adaptation
/// - Derived from `methodForRep` enum in the `libsixel` C library.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum RepColor {
    /// Choose automatically the method for selecting representative color from each box.
    /// (default)
    #[default]
    Auto,
    /// Choose the center of the box.
    Center,
    /// Choose the average of all the colors in the box (specified in Heckbert's paper).
    AverageColors,
    /// Choose the average of all the pixels in the box.
    AveragePixels,
}
#[rustfmt::skip]
impl ConstDefault for RepColor { const DEFAULT: Self = Self::Auto; }

/// Method of diffusion.
///
/// # Adaptation
/// - Derived from `methodForDiffuse` enum in the `libsixel` C library.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum SixelDiffusion {
    /// Choose diffusion type automatically.
    #[default]
    Auto = 0,
    /// Do not diffuse.
    None = 1,
    /// Diffuse with Bill Atkinson's method.
    Atkinson = 2,
    /// Diffuse with Floyd-Steinberg method.
    FS = 3,
    /// Diffuse with Jarvis, Judice & Ninke method.
    JaJuNi = 4,
    /// Diffuse with Stucki's method.
    Stucki = 5,
    /// Diffuse with Burkes' method.
    Burkes = 6,
    /// Positionally stable arithmetic dither.
    ADither = 7,
    /// Positionally stable arithmetic xor based dither.
    XDither = 8,
}
#[rustfmt::skip]
impl ConstDefault for SixelDiffusion { const DEFAULT: Self = Self::Auto; }

/// Quality modes.
///
/// # Adaptation
/// Derived from `qualityMode` enum in the `libsixel` C library.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Quality {
    /// Choose quality mode automatically
    #[default]
    Auto,
    /// High quality palette construction
    High,
    /// Low quality palette construction
    Low,
    /// Full quality palette construction
    Full,
    /// High color
    HighColor,
}
#[rustfmt::skip]
impl ConstDefault for Quality { const DEFAULT: Self = Self::Auto; }

/// Pixel format type of input image.
///
/// # Adaptation
/// Derived from `pixelFormat` enum in the `libsixel` C library.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// RGB color 15bpp.
    RGB555 = 1,
    /// RGB color 16bpp.
    RGB565 = 2,
    /// RGB color 24bpp. (Default)
    #[default]
    RGB888 = 3, // for compatibility, the value must be 3.
    /// BGR color 15bpp.
    BGR555 = 4,
    /// BGR color 16bpp.
    BGR565 = 5,
    /// BGR color 24bpp.
    BGR888 = 6,
    /// ARGB color 32bpp.
    ARGB8888 = 0x10,
    /// RGBA color 32bpp.
    RGBA8888 = 0x11,
    /// ABGR color 32bpp.
    ABGR8888 = 0x12,
    /// BGRA color 32bpp.
    BGRA8888 = 0x13,
    /// Grayscale 1bpp.
    G1 = (1 << 6),
    /// Grayscale 2bpp.
    G2 = (1 << 6) | 0x01,
    /// Grayscale 4bpp.
    G4 = (1 << 6) | 0x02,
    /// Grayscale 8bpp.
    G8 = (1 << 6) | 0x03,
    /// AG grayscale 16bpp.
    AG88 = (1 << 6) | 0x13,
    /// GA grayscale 16bpp.
    GA88 = (1 << 6) | 0x23,
    /// Palette 1bpp.
    PAL1 = (1 << 7),
    /// Palette 2bpp.
    PAL2 = (1 << 7) | 0x01,
    /// Palette 4bpp.
    PAL4 = (1 << 7) | 0x02,
    /// Palette 8bpp.
    PAL8 = (1 << 7) | 0x03,
}
#[rustfmt::skip]
impl ConstDefault for PixelFormat { const DEFAULT: Self = Self::RGB888; }

impl PixelFormat {
    /// Returns the bits per pixel of the current format.
    #[rustfmt::skip]
    pub const fn bpp(self) -> usize {
        match self {
            PixelFormat::RGB555
            | PixelFormat::BGR555 => 15,
            PixelFormat::RGB565
            | PixelFormat::BGR565
            | PixelFormat::AG88
            | PixelFormat::GA88 => 16,
            PixelFormat::RGB888
            | PixelFormat::BGR888
            | PixelFormat::G8
            | PixelFormat::PAL8 => 24,
            PixelFormat::ARGB8888
            | PixelFormat::RGBA8888
            | PixelFormat::ABGR8888
            | PixelFormat::BGRA8888 => 32,
            PixelFormat::G1 | PixelFormat::PAL1 => 1,
            PixelFormat::G2 | PixelFormat::PAL2 => 2,
            PixelFormat::G4 | PixelFormat::PAL4 => 4,
        }
    }

    /// Returns the number of bytes required to store an image of the given dimensions,
    /// using the current pixel format.
    pub const fn required_bytes(self, width: i32, height: i32) -> usize {
        let total_bits = width as usize * height as usize * self.bpp();
        // FIX: devela::bytes_from_bits(total_bits)
        (total_bits + 7) / 8
    }
}

/// Policies of SIXEL encoding.
///
/// # Adaptation
/// Derived from `encodePolicy` enum in the `libsixel` C library.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EncodePolicy {
    /// Choose encoding policy automatically (default).
    #[default]
    Auto = 0,
    /// Encode as fast as possible.
    Fast = 1,
    /// Encode to as small sixel sequence as possible.
    Size = 2,
}
#[rustfmt::skip]
impl ConstDefault for EncodePolicy { const DEFAULT: Self = Self::Auto; }

/// Palette type.
///
/// # Adaptation
/// Derived from `paletteType` enum in the `libsixel` C library.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PaletteType {
    /// Choose palette type automatically.
    #[default]
    Auto,
    /// HLS colorspace.
    HLS,
    /// RGB colorspace.
    RGB,
}
#[rustfmt::skip]
impl ConstDefault for PaletteType { const DEFAULT: Self = Self::Auto; }

/// Loop mode.
///
/// # Adaptation
/// Derived from `loopControl` enum in the `libsixel` C library.
#[repr(u8)]
#[expect(dead_code, reason = "Only using `Auto` for now")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
enum Loop {
    /// Honer the setting of GIF header.
    #[default]
    Auto,
    /// Always enable loop.
    Force,
    /// Always disable loop.
    Disable,
}
#[rustfmt::skip]
impl ConstDefault for Loop { const DEFAULT: Self = Self::Auto; }

// /// Method of resampling.
// ///
// /// # Adaptation
// /// Derived from `methodForResampling` enum in the `libsixel` C library.
// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub enum ResampleMethod { // TODO:MAYBE
//     /// Use nearest neighbor method
//     Nearest,
//     /// Use guaussian filter
//     Gaussian,
//     /// Use hanning filter
//     Hanning,
//     /// Use hamming filter
//     Hamming,
//     /// Use bilinear filter
//     Bilinear,
//     /// Use welfilter
//     Welsh,
//     /// Use bicubic filter
//     Bicubic,
//     /// Use lanczos-2 filter
//     Lanczos2,
//     /// Use lanczos-3 filter
//     Lanczos3,
//     /// Use lanczos-4 filter
//     Lanczos4,
// }

// /// Image format
// ///
// /// # Adaptation
// /// Derived from `imageFormat` enum in the `libsixel` C library.
// #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
// enum Format { // TODO:MAYBE
//     GIF,   //         0x0 /* read only */
//     PNG,   //         0x1 /* read/write */
//     BMP,   //         0x2 /* read only */
//     JPG,   //         0x3 /* read only */
//     TGA,   //         0x4 /* read only */
//     WBMP,  //         0x5 /* read only with --with-gd configure option */
//     TIFF,  //         0x6 /* read only */
//     SIXEL, //         0x7 /* read only */
//     PNM,   //         0x8 /* read only */
//     GD2,   //         0x9 /* read only with --with-gd configure option */
//     PSD,   //         0xa /* read only */
//     HDR,   //         0xb /* read only */
// }

// /// Offset value of `PixelFormat`.
// ///
// /// # Adaptation
// /// Derived from `formatType` enum in the `libsixel` C library.
// #[repr(u8)]
// #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
// pub enum FormatType { // TODO:MAYBE
//     Color,     // 0
//     Grayscale, // (1 << 6)
//     Palette,   // (1 << 7)
// }
