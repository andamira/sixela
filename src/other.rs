// sixela::other

use crate::all::{sixel_output, DitherConf, SixelResult};
use devela::{String, ToString, Vec};

/// Writes a string of sixel data.
///
/// # Example
/// ```
/// # use sixela::*;
/// // 2x2 pixels (Red, Green, Blue, White)
/// const IMAGE_HEX: &[u8] = b"FF000000FF000000FFFFFFFF";
///                          //RRGGBBrrggbbRRGGBBrrggbb
///
/// println!("{}", sixel_string(
///     IMAGE_HEX,
///     2,
///     2,
///     PixelFormat::RGB888,
///     DiffusionMethod::Stucki,
///     MethodForLargest::Auto,
///     MethodForRep::Auto,
///     Quality::Auto
/// ).unwrap());
/// ```
#[expect(clippy::too_many_arguments)]
pub fn sixel_string(
    bytes: &[u8],
    width: i32,
    height: i32,
    pixelformat: PixelFormat,
    method_for_diffuse: DiffusionMethod,
    method_for_largest: MethodForLargest,
    method_for_rep: MethodForRep,
    quality_mode: Quality,
) -> SixelResult<String> {
    // IMPROVE:CHECK: we should make sure we receive positive values.
    let mut sixel_data: Vec<u8> = Vec::new(); // MAYBE with_capacity?

    let mut sixel_output = sixel_output::new(&mut sixel_data);
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

/// Method for finding the largest dimension for splitting,
/// and sorting by that component.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum MethodForLargest {
    /// Choose automatically the method for finding the largest dimension. (default)
    #[default]
    Auto,
    /// Simply comparing the range in RGB space.
    Norm,
    /// Transforming into luminosities before the comparison.
    Lum,
}

/// Method for choosing a color from the box.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum MethodForRep {
    /// Choose automatically the method for selecting representative color from each box.
    /// (default)
    #[default]
    Auto,
    /// Choose the center of the box.
    CenterBox,
    /// Choose the average all the color in the box (specified in Heckbert's paper).
    AverageColors,
    /// Choose the average all the pixels in the box.
    Pixels,
}

/// Method of diffusion.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum DiffusionMethod {
    /// choose diffusion type automatically.
    #[default]
    Auto = 0,
    /// Po not diffuse.
    None = 1,
    /// Piffuse with Bill Atkinson's method.
    Atkinson = 2,
    /// Piffuse with Floyd-Steinberg method.
    FS = 3,
    /// Piffuse with Jarvis, Judice & Ninke method.
    JaJuNi = 4,
    /// Piffuse with Stucki's method.
    Stucki = 5,
    /// Piffuse with Burkes' method.
    Burkes = 6,
    /// Positionally stable arithmetic dither.
    ADither = 7,
    /// Positionally stable arithmetic xor based dither.
    XDither = 8,
}

/// Quality modes.
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

/// Offset value of `PixelFormat`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FormatType {
    Color,     // 0
    Grayscale, // (1 << 6)
    Palette,   // (1 << 7)
}

/// Pixelformat type of input image
///
// NOTE: for compatibility, the value of PIXELFORAMT_COLOR_RGB888 must be 3
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    RGB555 = 1,             //   (SIXEL_FORMATTYPE_COLOR     | 0x01) /* 15bpp */
    RGB565 = 2,             //   (SIXEL_FORMATTYPE_COLOR     | 0x02) /* 16bpp */
    RGB888 = 3,             //   (SIXEL_FORMATTYPE_COLOR     | 0x03) /* 24bpp */
    BGR555 = 4,             //   (SIXEL_FORMATTYPE_COLOR     | 0x04) /* 15bpp */
    BGR565 = 5,             //   (SIXEL_FORMATTYPE_COLOR     | 0x05) /* 16bpp */
    BGR888 = 6,             //   (SIXEL_FORMATTYPE_COLOR     | 0x06) /* 24bpp */
    ARGB8888 = 0x10,        // (SIXEL_FORMATTYPE_COLOR     | 0x10) /* 32bpp */
    RGBA8888 = 0x11,        // (SIXEL_FORMATTYPE_COLOR     | 0x11) /* 32bpp */
    ABGR8888 = 0x12,        // (SIXEL_FORMATTYPE_COLOR     | 0x12) /* 32bpp */
    BGRA8888 = 0x13,        // (SIXEL_FORMATTYPE_COLOR     | 0x13) /* 32bpp */
    G1 = (1 << 6),          //       (SIXEL_FORMATTYPE_GRAYSCALE | 0x00) /* 1bpp grayscale */
    G2 = (1 << 6) | 0x01,   //       (SIXEL_FORMATTYPE_GRAYSCALE | 0x01) /* 2bpp grayscale */
    G4 = (1 << 6) | 0x02,   //       (SIXEL_FORMATTYPE_GRAYSCALE | 0x02) /* 4bpp grayscale */
    G8 = (1 << 6) | 0x03,   //       (SIXEL_FORMATTYPE_GRAYSCALE | 0x03) /* 8bpp grayscale */
    AG88 = (1 << 6) | 0x13, //     (SIXEL_FORMATTYPE_GRAYSCALE | 0x13) /* 16bpp gray+alpha */
    GA88 = (1 << 6) | 0x23, //     (SIXEL_FORMATTYPE_GRAYSCALE | 0x23) /* 16bpp gray+alpha */
    PAL1 = (1 << 7),        //     (SIXEL_FORMATTYPE_PALETTE   | 0x00) /* 1bpp palette */
    PAL2 = (1 << 7) | 0x01, //     (SIXEL_FORMATTYPE_PALETTE   | 0x01) /* 2bpp palette */
    PAL4 = (1 << 7) | 0x02, //     (SIXEL_FORMATTYPE_PALETTE   | 0x02) /* 4bpp palette */
    PAL8 = (1 << 7) | 0x03, //     (SIXEL_FORMATTYPE_PALETTE   | 0x03) /* 8bpp palette */
}

/// TODO
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PaletteType {
    /// choose palette type automatically
    Auto,
    /// HLS colorspace
    HLS,
    /// RGB colorspace
    RGB,
}

/// policies of SIXEL encoding
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

/// TODO
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResampleMethod {
    /// Use nearest neighbor method
    NEAREST,
    /// Use guaussian filter
    GAUSSIAN,
    /// Use hanning filter
    HANNING,
    /// Use hamming filter
    HAMMING,
    /// Use bilinear filter
    BILINEAR,
    /// Use welfilter
    WELSH,
    /// Use bicubic filter
    BICUBIC,
    /// Use lanczos-2 filter
    LANCZOS2,
    /// Use lanczos-3 filter
    LANCZOS3,
    /// Use lanczos-4 filter
    LANCZOS4,
}

// Image format
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Format {
    GIF,   //         0x0 /* read only */
    PNG,   //         0x1 /* read/write */
    BMP,   //         0x2 /* read only */
    JPG,   //         0x3 /* read only */
    TGA,   //         0x4 /* read only */
    WBMP,  //         0x5 /* read only with --with-gd configure option */
    TIFF,  //         0x6 /* read only */
    SIXEL, //         0x7 /* read only */
    PNM,   //         0x8 /* read only */
    GD2,   //         0x9 /* read only with --with-gd configure option */
    PSD,   //         0xa /* read only */
    HDR,   //         0xb /* read only */
}

// Loop mode
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
enum Loop {
    /// honer the setting of GIF header
    #[default]
    Auto,
    /// always enable loop
    Force,
    /// always disable loop
    Disable,
}
