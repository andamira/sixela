// sixela::output

use crate::{EncodePolicy, PaletteType};
use devela::{sys::Write as IoWrite, String, Vec};

/// Represents a single sixel tile with color and spatial properties.
///
/// Holds the palette index, x-coordinates, and a map of color data
/// for efficient rendering of individual sixel tiles.
///
/// # Adaptation
/// - Derived from `sixe_node` struct in the `libsixel` C library.
#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct SixelNode {
    /// Index of the color in the palette.
    pub pal: i32,
    /// Start x-coordinate of the tile.
    pub sx: i32,
    /// End x-coordinate of the tile.
    pub mx: i32,
    /// Color data map for the tile.
    pub map: Vec<u8>,
}

/// Handles sixel data output to a specified writer destination.
///
/// Abstracts over writing sixel-encoded data,
/// supporting various output targets such as files or terminal streams.
///
/// # Adaptation
/// - Derived from `sixel_output` struct in the `libsixel` C library.
#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct SixelOutput<W: IoWrite> {
    /* private compatiblity flags */
    /// Indicates 8-bit terminal support.
    ///
    /// `false` for 7-bit, `true` for 8-bit.
    pub(crate) has_8bit_control: bool,

    /// Sixel scrolling support flag.
    ///
    /// `false` if terminal supports scrolling, `true` if not.
    pub(crate) has_sixel_scrolling: bool,

    /// Argument limit for repeat introducer (DECGRI).
    ///
    /// `false` if limited to 255, `true` if unlimited.
    pub(crate) has_gri_arg_limit: bool,

    /// DECSDM (CSI ? 80 h) sixel scrolling glitch flag.
    ///
    /// `false` enables sixel scrolling, `true` disables it.
    pub(crate) has_sdm_glitch: bool,

    /// Flag to skip DCS envelope handling.
    ///
    /// `false` to process, `true` to skip.
    pub(crate) skip_dcs_envelope: bool,

    /* public fields */
    /// Palette selection mode.
    pub palette_type: PaletteType,

    /// Writer for output, managing data destination.
    pub fn_write: W,

    /// Last saved pixel value.
    pub save_pixel: u8,
    /// Count of consecutive saved pixels.
    pub save_count: i32,
    /// Currently active palette index.
    pub active_palette: i32,

    /// Collection of sixel nodes for dithering.
    pub nodes: Vec<SixelNode>,

    /// Flag to allow penetration of the multiplexer.
    pub penetrate_multiplexer: bool,
    /// Policy for encoding decisions.
    pub encode_policy: EncodePolicy,

    /// Buffer for output data.
    pub buffer: String,
}

impl<W: IoWrite> SixelOutput<W> {
    /// Packet size limit.
    pub(crate) const PACKET_SIZE: usize = 16_384;

    /// Create new output context object
    #[inline]
    pub fn new(fn_write: W) -> Self {
        Self {
            has_8bit_control: false,
            has_sdm_glitch: false,
            has_gri_arg_limit: true,
            skip_dcs_envelope: false,
            palette_type: PaletteType::Auto,
            fn_write,
            save_pixel: 0,
            save_count: 0,
            active_palette: -1,
            nodes: Vec::new(),
            penetrate_multiplexer: false,
            encode_policy: EncodePolicy::Auto,
            has_sixel_scrolling: false,
            buffer: String::new(),
        }
    }

    /// get 8bit output mode which indicates whether it uses C1 control characters.
    #[inline]
    pub fn get_8bit_availability(&self) -> bool {
        self.has_8bit_control
    }

    /// Set 8bit output mode state.
    #[inline]
    pub fn set_8bit_availability(&mut self, availability: bool) {
        self.has_8bit_control = availability;
    }

    /// Set limit for repeat introducer (DECGRI).
    ///
    /// `false` if limited to 255, `true` if unlimited.
    #[inline]
    pub fn set_gri_arg_limit(&mut self, value: bool) {
        self.has_gri_arg_limit = value;
    }

    /// Set GNU Screen penetration.
    #[inline]
    pub fn set_penetrate_multiplexer(&mut self, penetrate: bool) {
        self.penetrate_multiplexer = penetrate;
    }

    /// Set whether we skip DCS envelope.
    #[inline]
    pub fn set_skip_dcs_envelope(&mut self, skip: bool) {
        self.skip_dcs_envelope = skip;
    }

    /// Set the palette type.
    #[inline]
    pub fn set_palette_type(&mut self, palettetype: PaletteType) {
        self.palette_type = palettetype;
    }

    /// Set the encoding policy.
    #[inline]
    pub fn set_encode_policy(&mut self, encode_policy: EncodePolicy) {
        self.encode_policy = encode_policy;
    }
}
