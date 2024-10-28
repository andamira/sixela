// sixela::output::builder

use crate::{
    DitherConf, EncodePolicy, MethodForLargest, MethodForRep, PixelFormat, Quality, SixelDiffusion,
    SixelOutput, SixelResult,
};
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
///     IMAGE_HEX, 2, 2,
///     PixelFormat::RGB888,
///     SixelDiffusion::Stucki,
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
    method_for_diffuse: SixelDiffusion,
    method_for_largest: MethodForLargest,
    method_for_rep: MethodForRep,
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
