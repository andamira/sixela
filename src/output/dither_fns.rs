// sixela::output::dither_fns
//
// TOC
// - fn sixel_apply_15bpp_dither
// - fn dither_func_none
// - fn dither_func_fs
// - fn dither_func_atkinson
// - fn dither_func_jajuni
// - fn dither_func_stucki
// - fn dither_func_burkes
// - fn dither_func_a_dither
// - fn dither_func_x_dither

use crate::SixelDiffusion;

/// TODO
pub(super) fn sixel_apply_15bpp_dither(
    pixels: &mut [u8],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    method_for_diffuse: SixelDiffusion,
) {
    match method_for_diffuse {
        SixelDiffusion::None | SixelDiffusion::Auto => {
            dither_func_none(pixels, width);
        }
        SixelDiffusion::Atkinson => {
            if x < width - 2 && y < height - 2 {
                dither_func_atkinson(pixels, width);
            }
        }
        SixelDiffusion::FS => {
            if x < width - 1 && y < height - 1 {
                dither_func_fs(pixels, width);
            }
        }
        SixelDiffusion::JaJuNi => {
            if x < width - 2 && y < height - 2 {
                dither_func_jajuni(pixels, width);
            }
        }
        SixelDiffusion::Stucki => {
            if x < width - 2 && y < height - 2 {
                dither_func_stucki(pixels, width);
            }
        }
        SixelDiffusion::Burkes => {
            if x < width - 2 && y < height - 1 {
                dither_func_burkes(pixels, width);
            }
        }
        SixelDiffusion::ADither => {
            dither_func_a_dither(pixels, width, x, y);
        }
        SixelDiffusion::XDither => {
            dither_func_x_dither(pixels, width, x, y);
        }
    }
}

/// No dithering
#[inline]
fn dither_func_none(_data: &mut [u8], _width: i32) {}

/// Floyd Steinberg dithering
///
/// ```txt
///         curr    7/16
/// 3/16    5/48    1/16
/// ```
fn dither_func_fs(data: &mut [u8], width: i32) {
    let error_r = data[0] as i32 & 0x7;
    let error_g = data[1] as i32 & 0x7;
    let error_b = data[2] as i32 & 0x7;
    let width = width as usize;
    let mut r = data[3 + 0] as i32 + ((error_r * 5) >> 4);
    let mut g = data[3 + 1] as i32 + ((error_g * 5) >> 4);
    let mut b = data[3 + 2] as i32 + ((error_b * 5) >> 4);
    data[3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[width * 3 - 3 + 0] as i32 + ((error_r * 3) >> 4);
    g = data[width * 3 - 3 + 1] as i32 + ((error_g * 3) >> 4);
    b = data[width * 3 - 3 + 2] as i32 + ((error_b * 3) >> 4);
    data[width * 3 - 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[width * 3 - 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[width * 3 - 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[width * 3 + 0] as i32 + ((error_r * 5) >> 4);
    g = data[width * 3 + 1] as i32 + ((error_g * 5) >> 4);
    b = data[width * 3 + 2] as i32 + ((error_b * 5) >> 4);
    data[width * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[width * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[width * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
}

/// Atkinson's dithering
///
/// ```txt
///         curr    1/8    1/8
///  1/8     1/8    1/8
/// ```
fn dither_func_atkinson(data: &mut [u8], width: i32) {
    let mut error_r = data[0] as i32 & 0x7;
    let mut error_g = data[1] as i32 & 0x7;
    let mut error_b = data[2] as i32 & 0x7;
    error_r += 4;
    error_g += 4;
    error_b += 4;
    let width = width as usize;

    let mut r = data[(width * 0 + 1) * 3 + 0] as i32 + (error_r >> 3);
    let mut g = data[(width * 0 + 1) * 3 + 1] as i32 + (error_g >> 3);
    let mut b = data[(width * 0 + 1) * 3 + 2] as i32 + (error_b >> 3);
    data[(width * 0 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 0 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 0 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 0 + 2) * 3 + 0] as i32 + (error_r >> 3);
    g = data[(width * 0 + 2) * 3 + 1] as i32 + (error_g >> 3);
    b = data[(width * 0 + 2) * 3 + 2] as i32 + (error_b >> 3);
    data[(width * 0 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 0 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 0 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 - 1) * 3 + 0] as i32 + (error_r >> 3);
    g = data[(width * 1 - 1) * 3 + 1] as i32 + (error_g >> 3);
    b = data[(width * 1 - 1) * 3 + 2] as i32 + (error_b >> 3);
    data[(width * 1 - 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 - 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 - 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 0) * 3 + 0] as i32 + (error_r >> 3);
    g = data[(width * 1 + 0) * 3 + 1] as i32 + (error_g >> 3);
    b = data[(width * 1 + 0) * 3 + 2] as i32 + (error_b >> 3);
    data[(width * 1 + 0) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 0) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 0) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 1) * 3 + 0] as i32 + (error_r >> 3);
    g = data[(width * 1 + 1) * 3 + 1] as i32 + (error_g >> 3);
    b = data[(width * 1 + 1) * 3 + 2] as i32 + (error_b >> 3);
    data[(width * 1 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 + 0) * 3 + 0] as i32 + (error_r >> 3);
    g = data[(width * 2 + 0) * 3 + 1] as i32 + (error_g >> 3);
    b = data[(width * 2 + 0) * 3 + 2] as i32 + (error_b >> 3);
    data[(width * 2 + 0) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 + 0) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 + 0) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
}

/// Jarvis, Judice & Ninke dithering
///
/// ```txt
///                 curr    7/48    5/48
/// 3/48    5/48    7/48    5/48    3/48
/// 1/48    3/48    5/48    3/48    1/48
/// ```
fn dither_func_jajuni(data: &mut [u8], width: i32) {
    let mut error_r = data[0] as i32 & 0x7;
    let mut error_g = data[1] as i32 & 0x7;
    let mut error_b = data[2] as i32 & 0x7;
    error_r += 4;
    error_g += 4;
    error_b += 4;
    let width = width as usize;

    let mut r = data[(width * 0 + 1) * 3 + 0] as i32 + (error_r * 7 / 48);
    let mut g = data[(width * 0 + 1) * 3 + 1] as i32 + (error_g * 7 / 48);
    let mut b = data[(width * 0 + 1) * 3 + 2] as i32 + (error_b * 7 / 48);
    data[(width * 0 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 0 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 0 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 0 + 2) * 3 + 0] as i32 + (error_r * 5 / 48);
    g = data[(width * 0 + 2) * 3 + 1] as i32 + (error_g * 5 / 48);
    b = data[(width * 0 + 2) * 3 + 2] as i32 + (error_b * 5 / 48);
    data[(width * 0 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 0 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 0 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 - 2) * 3 + 0] as i32 + (error_r * 3 / 48);
    g = data[(width * 1 - 2) * 3 + 1] as i32 + (error_g * 3 / 48);
    b = data[(width * 1 - 2) * 3 + 2] as i32 + (error_b * 3 / 48);
    data[(width * 1 - 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 - 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 - 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 - 1) * 3 + 0] as i32 + (error_r * 5 / 48);
    g = data[(width * 1 - 1) * 3 + 1] as i32 + (error_g * 5 / 48);
    b = data[(width * 1 - 1) * 3 + 2] as i32 + (error_b * 5 / 48);
    data[(width * 1 - 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 - 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 - 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 0) * 3 + 0] as i32 + (error_r * 7 / 48);
    g = data[(width * 1 + 0) * 3 + 1] as i32 + (error_g * 7 / 48);
    b = data[(width * 1 + 0) * 3 + 2] as i32 + (error_b * 7 / 48);
    data[(width * 1 + 0) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 0) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 0) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 1) * 3 + 0] as i32 + (error_r * 5 / 48);
    g = data[(width * 1 + 1) * 3 + 1] as i32 + (error_g * 5 / 48);
    b = data[(width * 1 + 1) * 3 + 2] as i32 + (error_b * 5 / 48);
    data[(width * 1 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 2) * 3 + 0] as i32 + (error_r * 3 / 48);
    g = data[(width * 1 + 2) * 3 + 1] as i32 + (error_g * 3 / 48);
    b = data[(width * 1 + 2) * 3 + 2] as i32 + (error_b * 3 / 48);
    data[(width * 1 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 - 2) * 3 + 0] as i32 + (error_r * 1 / 48);
    g = data[(width * 2 - 2) * 3 + 1] as i32 + (error_g * 1 / 48);
    b = data[(width * 2 - 2) * 3 + 2] as i32 + (error_b * 1 / 48);
    data[(width * 2 - 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 - 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 - 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 - 1) * 3 + 0] as i32 + (error_r * 3 / 48);
    g = data[(width * 2 - 1) * 3 + 1] as i32 + (error_g * 3 / 48);
    b = data[(width * 2 - 1) * 3 + 2] as i32 + (error_b * 3 / 48);
    data[(width * 2 - 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 - 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 - 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 + 0) * 3 + 0] as i32 + (error_r * 5 / 48);
    g = data[(width * 2 + 0) * 3 + 1] as i32 + (error_g * 5 / 48);
    b = data[(width * 2 + 0) * 3 + 2] as i32 + (error_b * 5 / 48);
    data[(width * 2 + 0) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 + 0) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 + 0) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 + 1) * 3 + 0] as i32 + (error_r * 3 / 48);
    g = data[(width * 2 + 1) * 3 + 1] as i32 + (error_g * 3 / 48);
    b = data[(width * 2 + 1) * 3 + 2] as i32 + (error_b * 3 / 48);
    data[(width * 2 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 + 2) * 3 + 0] as i32 + (error_r * 1 / 48);
    g = data[(width * 2 + 2) * 3 + 1] as i32 + (error_g * 1 / 48);
    b = data[(width * 2 + 2) * 3 + 2] as i32 + (error_b * 1 / 48);
    data[(width * 2 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
}

/// Stucki's dithering
///
/// ```txt
///                  curr    8/48    4/48
///  2/48    4/48    8/48    4/48    2/48
///  1/48    2/48    4/48    2/48    1/48
/// ```
fn dither_func_stucki(data: &mut [u8], width: i32) {
    let mut error_r = data[0] as i32 & 0x7;
    let mut error_g = data[1] as i32 & 0x7;
    let mut error_b = data[2] as i32 & 0x7;
    error_r += 4;
    error_g += 4;
    error_b += 4;
    let width = width as usize;

    let mut r = data[(width * 0 + 1) * 3 + 0] as i32 + (error_r * 8 / 48);
    let mut g = data[(width * 0 + 1) * 3 + 1] as i32 + (error_g * 8 / 48);
    let mut b = data[(width * 0 + 1) * 3 + 2] as i32 + (error_b * 8 / 48);
    data[(width * 0 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 0 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 0 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 0 + 2) * 3 + 0] as i32 + (error_r * 4 / 48);
    g = data[(width * 0 + 2) * 3 + 1] as i32 + (error_g * 4 / 48);
    b = data[(width * 0 + 2) * 3 + 2] as i32 + (error_b * 4 / 48);
    data[(width * 0 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 0 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 0 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 - 2) * 3 + 0] as i32 + (error_r * 2 / 48);
    g = data[(width * 1 - 2) * 3 + 1] as i32 + (error_g * 2 / 48);
    b = data[(width * 1 - 2) * 3 + 2] as i32 + (error_b * 2 / 48);
    data[(width * 1 - 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 - 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 - 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 - 1) * 3 + 0] as i32 + (error_r * 4 / 48);
    g = data[(width * 1 - 1) * 3 + 1] as i32 + (error_g * 4 / 48);
    b = data[(width * 1 - 1) * 3 + 2] as i32 + (error_b * 4 / 48);
    data[(width * 1 - 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 - 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 - 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 0) * 3 + 0] as i32 + (error_r * 8 / 48);
    g = data[(width * 1 + 0) * 3 + 1] as i32 + (error_g * 8 / 48);
    b = data[(width * 1 + 0) * 3 + 2] as i32 + (error_b * 8 / 48);
    data[(width * 1 + 0) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 0) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 0) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 1) * 3 + 0] as i32 + (error_r * 4 / 48);
    g = data[(width * 1 + 1) * 3 + 1] as i32 + (error_g * 4 / 48);
    b = data[(width * 1 + 1) * 3 + 2] as i32 + (error_b * 4 / 48);
    data[(width * 1 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 2) * 3 + 0] as i32 + (error_r * 2 / 48);
    g = data[(width * 1 + 2) * 3 + 1] as i32 + (error_g * 2 / 48);
    b = data[(width * 1 + 2) * 3 + 2] as i32 + (error_b * 2 / 48);
    data[(width * 1 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 - 2) * 3 + 0] as i32 + (error_r * 1 / 48);
    g = data[(width * 2 - 2) * 3 + 1] as i32 + (error_g * 1 / 48);
    b = data[(width * 2 - 2) * 3 + 2] as i32 + (error_b * 1 / 48);
    data[(width * 2 - 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 - 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 - 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 - 1) * 3 + 0] as i32 + (error_r * 2 / 48);
    g = data[(width * 2 - 1) * 3 + 1] as i32 + (error_g * 2 / 48);
    b = data[(width * 2 - 1) * 3 + 2] as i32 + (error_b * 2 / 48);
    data[(width * 2 - 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 - 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 - 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 + 0) * 3 + 0] as i32 + (error_r * 4 / 48);
    g = data[(width * 2 + 0) * 3 + 1] as i32 + (error_g * 4 / 48);
    b = data[(width * 2 + 0) * 3 + 2] as i32 + (error_b * 4 / 48);
    data[(width * 2 + 0) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 + 0) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 + 0) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 + 1) * 3 + 0] as i32 + (error_r * 2 / 48);
    g = data[(width * 2 + 1) * 3 + 1] as i32 + (error_g * 2 / 48);
    b = data[(width * 2 + 1) * 3 + 2] as i32 + (error_b * 2 / 48);
    data[(width * 2 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 2 + 2) * 3 + 0] as i32 + (error_r * 1 / 48);
    g = data[(width * 2 + 2) * 3 + 1] as i32 + (error_g * 1 / 48);
    b = data[(width * 2 + 2) * 3 + 2] as i32 + (error_b * 1 / 48);
    data[(width * 2 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 2 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 2 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
}

/// Burkes' Method
///
/// ```txt
///                  curr    4/16    2/16
///  1/16    2/16    4/16    2/16    1/16
/// ```
fn dither_func_burkes(data: &mut [u8], width: i32) {
    let mut error_r = data[0] as i32 & 0x7;
    let mut error_g = data[1] as i32 & 0x7;
    let mut error_b = data[2] as i32 & 0x7;
    error_r += 2;
    error_g += 2;
    error_b += 2;
    let width = width as usize;

    let mut r = data[(width * 0 + 1) * 3 + 0] as i32 + (error_r * 4 / 16);
    let mut g = data[(width * 0 + 1) * 3 + 1] as i32 + (error_g * 4 / 16);
    let mut b = data[(width * 0 + 1) * 3 + 2] as i32 + (error_b * 4 / 16);
    data[(width * 0 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 0 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 0 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 0 + 2) * 3 + 0] as i32 + (error_r * 2 / 16);
    g = data[(width * 0 + 2) * 3 + 1] as i32 + (error_g * 2 / 16);
    b = data[(width * 0 + 2) * 3 + 2] as i32 + (error_b * 2 / 16);
    data[(width * 0 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 0 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 0 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 - 2) * 3 + 0] as i32 + (error_r * 1 / 16);
    g = data[(width * 1 - 2) * 3 + 1] as i32 + (error_g * 1 / 16);
    b = data[(width * 1 - 2) * 3 + 2] as i32 + (error_b * 1 / 16);
    data[(width * 1 - 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 - 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 - 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 - 1) * 3 + 0] as i32 + (error_r * 2 / 16);
    g = data[(width * 1 - 1) * 3 + 1] as i32 + (error_g * 2 / 16);
    b = data[(width * 1 - 1) * 3 + 2] as i32 + (error_b * 2 / 16);
    data[(width * 1 - 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 - 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 - 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 0) * 3 + 0] as i32 + (error_r * 4 / 16);
    g = data[(width * 1 + 0) * 3 + 1] as i32 + (error_g * 4 / 16);
    b = data[(width * 1 + 0) * 3 + 2] as i32 + (error_b * 4 / 16);
    data[(width * 1 + 0) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 0) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 0) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 1) * 3 + 0] as i32 + (error_r * 2 / 16);
    g = data[(width * 1 + 1) * 3 + 1] as i32 + (error_g * 2 / 16);
    b = data[(width * 1 + 1) * 3 + 2] as i32 + (error_b * 2 / 16);
    data[(width * 1 + 1) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 1) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 1) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
    r = data[(width * 1 + 2) * 3 + 0] as i32 + (error_r * 1 / 16);
    g = data[(width * 1 + 2) * 3 + 1] as i32 + (error_g * 1 / 16);
    b = data[(width * 1 + 2) * 3 + 2] as i32 + (error_b * 1 / 16);
    data[(width * 1 + 2) * 3 + 0] = if r > 0xff { 0xff } else { r as u8 };
    data[(width * 1 + 2) * 3 + 1] = if g > 0xff { 0xff } else { g as u8 };
    data[(width * 1 + 2) * 3 + 2] = if b > 0xff { 0xff } else { b as u8 };
}

/// TODO
fn dither_func_a_dither(data: &mut [u8], _width: i32, x: i32, y: i32) {
    for c in 0..3 {
        let mask = (((x + c * 17) + y * 236) * 119) & 255;
        let mask = (mask - 128) / 256;
        let value = data[c as usize] as i32 + mask;
        data[c as usize] = value.clamp(0, 255) as u8;
    }
}

/// TODO
fn dither_func_x_dither(data: &mut [u8], _width: i32, x: i32, y: i32) {
    for c in 0..3 {
        let mask = ((((x + c * 17) ^ y) * 236) * 1234) & 511;
        let mask = (mask - 128) / 512;
        let value = data[c as usize] as i32 + mask;
        data[c as usize] = value.clamp(0, 255) as u8;
    }
}
