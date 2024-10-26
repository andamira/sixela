// sixela::lib
//
//! Sixel in pure rust.
//

//* global config *//
//
// lints
#![deny(
    // WAIT: [lazy_type_alias](https://github.com/rust-lang/rust/issues/112792)
    type_alias_bounds, // detects bounds in type aliases
    unsafe_op_in_unsafe_fn, // unsafe operations in unsafe functions without explicit unsafe block
    clippy::missing_safety_doc, // deny if there's no # Safety section in public unsafe fns
)]
#![warn(
    // missing_docs, // missing docs for public items
    clippy::all, // (the default set of clippy lints)
    // a selection from clippy::pedantic:
    clippy::bool_to_int_with_if, // using an if statement to convert a bool to an int
    clippy::cloned_instead_of_copied, // usage of cloned() where copied() could be used
    clippy::default_union_representation, // union declared without #[repr(C)]
    clippy::empty_structs_with_brackets, // structs without fields, with brackets
    clippy::enum_glob_use, // checks for `use Enum::*`
    clippy::if_then_some_else_none, // if-else that could be written using bool::then[_some]
    clippy::ignored_unit_patterns, // Checks for usage of _ in patterns of type ()
    clippy::float_cmp, // (in-)equality comparisons on floating-point values
    clippy::float_cmp_const, // (in-)equality comparisons on const floating-point values
    clippy::manual_let_else, // cases where let...else could be used
    clippy::manual_string_new, // usage of "" to create a String
    clippy::map_unwrap_or, // usage of result|option.map(_).unwrap_or[_else](_)
    clippy::ptr_cast_constness, // as casts between raw pointers that change their constness
    clippy::same_functions_in_if_condition, // consecutive ifs with the same function call
    clippy::semicolon_if_nothing_returned, // expression returns () not followed by a semicolon
    clippy::single_match_else, // matches with two arms where an if let else will usually suffice
    clippy::trivially_copy_pass_by_ref, // fns with ref args that could be passed by value
    clippy::unnested_or_patterns, // unnested or-patterns, (Some(a)|Some(b) vs Some(a|b))
    clippy::unreadable_literal, //  long integral does not contain underscores
)]
#![allow(
    clippy::identity_op, // * 1
    clippy::erasing_op,  // * 0
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    clippy::upper_case_acronyms,
    dead_code,
)]
//
// nightly, safety, environment
#![cfg_attr(feature = "nightly", feature(doc_cfg))]
#![cfg_attr(feature = "safe", forbid(unsafe_code))]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

// safeguarding: environment, safety
#[cfg(all(feature = "std", feature = "no_std"))]
compile_error!("You can't enable the `std` and `no_std` features at the same time.");
#[cfg(all(feature = "safe", feature = "unsafe"))]
compile_error!("You can't enable `safe` and `unsafe*` features at the same time.");

use devela::{String, ToString, Vec};

mod error;
pub use error::*;

pub mod dither;
pub mod output;
pub mod pixelformat;
pub mod quant;
pub mod tosixel;

use dither::sixel_dither;
use output::sixel_output;

/* limitations */
const SIXEL_OUTPUT_PACKET_SIZE: usize = 16_384;
const SIXEL_PALETTE_MIN: usize = 2;
const SIXEL_PALETTE_MAX: usize = 256;
const SIXEL_USE_DEPRECATED_SYMBOLS: usize = 1;
const SIXEL_ALLOCATE_BYTES_MAX: usize = 10_248 * 1_024 * 128; /* up to 128M */
const SIXEL_WIDTH_LIMIT: usize = 1_000_000;
const SIXEL_HEIGHT_LIMIT: usize = 1_000_000;

/* loader settings */
const SIXEL_DEFALUT_GIF_DELAY: usize = 1;

/// method for finding the largest dimension for splitting,
/// and sorting by that component
#[derive(Clone, Copy)]
pub enum MethodForLargest {
    /// choose automatically the method for finding the largest dimension
    Auto,
    /// simply comparing the range in RGB space
    Norm,
    /// transforming into luminosities before the comparison
    Lum,
}

/// method for choosing a color from the box
#[derive(Clone, Copy)]
pub enum MethodForRep {
    /// choose automatically the method for selecting
    /// representative color from each box
    Auto,
    /// choose the center of the box
    CenterBox,
    /// choose the average all the color in the box (specified in Heckbert's paper)
    AverageColors,
    /// choose the average all the pixels in the box
    Pixels,
}

#[derive(Clone, Copy)]
pub enum DiffusionMethod {
    /// choose diffusion type automatically
    Auto = 0,
    /// don't diffuse
    None = 1,
    /// diffuse with Bill Atkinson's method
    Atkinson = 2,
    /// diffuse with Floyd-Steinberg method
    FS = 3,
    /// diffuse with Jarvis, Judice & Ninke method
    JaJuNi = 4,
    /// diffuse with Stucki's method
    Stucki = 5,
    /// diffuse with Burkes' method
    Burkes = 6,
    /// positionally stable arithmetic dither
    ADither = 7,
    /// positionally stable arithmetic xor based dither
    XDither = 8,
}

/// quality modes
#[derive(Clone, Copy)]
pub enum Quality {
    /// choose quality mode automatically
    AUTO,
    /// high quality palette construction
    HIGH,
    /// low quality palette construction
    LOW,
    /// full quality palette construction
    FULL,
    /// high color
    HIGHCOLOR,
}

/* built-in dither */
#[derive(Clone, Copy)]
pub enum BuiltinDither {
    /// monochrome terminal with dark background
    MonoDark,
    /// monochrome terminal with light background
    MonoLight,
    /// x
    /// term 16color
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

/// offset value of pixelFormat
pub enum FormatType {
    COLOR,     // 0
    GRAYSCALE, // (1 << 6)
    PALETTE,   //    (1 << 7)
}

/// pixelformat type of input image
/// NOTE: for compatibility, the value of PIXELFORAMT_COLOR_RGB888 must be 3
#[derive(Clone, Copy, PartialEq)]
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

pub enum PaletteType {
    /// choose palette type automatically
    Auto,
    /// HLS colorspace
    HLS,
    /// RGB colorspace
    RGB,
}

/// policies of SIXEL encoding
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncodePolicy {
    /// choose encoding policy automatically
    AUTO = 0,
    /// encode as fast as possible
    FAST = 1,
    /// encode to as small sixel sequence as possible
    SIZE = 2,
}

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
    /// Use welsh filter
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
/* image format */
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

/* loop mode */
enum Loop {
    /// honer the setting of GIF header
    AUTO,
    /// always enable loop
    FORCE,
    /// always disable loop
    DISABLE,
}

#[allow(clippy::too_many_arguments)]
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
    let mut sixel_data: Vec<u8> = Vec::new();

    let mut sixel_output = sixel_output::new(&mut sixel_data);
    sixel_output.set_encode_policy(EncodePolicy::AUTO);
    let mut sixel_dither = sixel_dither::new(256).unwrap();

    sixel_dither.set_optimize_palette(true);

    sixel_dither.initialize(
        bytes,
        width,
        height,
        pixelformat,
        method_for_largest,
        method_for_rep,
        quality_mode,
    )?;
    sixel_dither.set_pixelformat(pixelformat);
    sixel_dither.set_diffusion_type(method_for_diffuse);

    let mut bytes = bytes.to_vec();
    sixel_output.encode(&mut bytes, width, height, 0, &mut sixel_dither)?;

    Ok(String::from_utf8_lossy(&sixel_data).to_string())
}
