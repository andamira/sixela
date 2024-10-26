// sixela::output

use crate::{EncodePolicy, PaletteType};
use devela::{sys::Write as IoWrite, String, Vec};

#[derive(Default, PartialEq)]
pub struct sixel_node {
    pub pal: i32,
    pub sx: i32,
    pub mx: i32,
    pub map: Vec<u8>,
}

pub struct sixel_output<W: IoWrite> {
    /* compatiblity flags */

    /* 0: 7bit terminal,
     * 1: 8bit terminal */
    pub(crate) has_8bit_control: bool,

    /* 0: the terminal has sixel scrolling
     * 1: the terminal does not have sixel scrolling */
    pub(crate) has_sixel_scrolling: bool,

    /* 1: the argument of repeat introducer(DECGRI) is not limitted
    0: the argument of repeat introducer(DECGRI) is limitted 255 */
    pub(crate) has_gri_arg_limit: bool,

    /* 0: DECSDM set (CSI ? 80 h) enables sixel scrolling
    1: DECSDM set (CSI ? 80 h) disables sixel scrolling */
    pub(crate) has_sdm_glitch: bool,

    /* 0: do not skip DCS envelope
     * 1: skip DCS envelope */
    pub(crate) skip_dcs_envelope: bool,

    /* PALETTETYPE_AUTO: select palette type automatically
     * PALETTETYPE_HLS : HLS color space
     * PALETTETYPE_RGB : RGB color space */
    pub palette_type: PaletteType,

    pub fn_write: W,

    pub save_pixel: u8,
    pub save_count: i32,
    pub active_palette: i32,

    pub nodes: Vec<sixel_node>,

    pub penetrate_multiplexer: bool,
    pub encode_policy: EncodePolicy,

    pub buffer: String,
}

impl<W: IoWrite> sixel_output<W> {
    /// create new output context object
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

    /// get 8bit output mode which indicates whether it uses C1 control characters
    pub fn get_8bit_availability(&self) -> bool {
        self.has_8bit_control
    }

    /// set 8bit output mode state
    pub fn set_8bit_availability(&mut self, availability: bool) {
        self.has_8bit_control = availability;
    }

    /// set whether limit arguments of DECGRI('!') to 255
    ///   /* 0: don't limit arguments of DECGRI
    /// 1: limit arguments of DECGRI to 255 */
    pub fn set_gri_arg_limit(&mut self, value: bool) {
        self.has_gri_arg_limit = value;
    }

    /// set GNU Screen penetration feature enable or disable
    pub fn set_penetrate_multiplexer(&mut self, penetrate: bool) {
        self.penetrate_multiplexer = penetrate;
    }

    /// set whether we skip DCS envelope
    pub fn set_skip_dcs_envelope(&mut self, skip: bool) {
        self.skip_dcs_envelope = skip;
    }

    /// set palette type: RGB or HLS
    pub fn set_palette_type(&mut self, palettetype: PaletteType) {
        self.palette_type = palettetype;
    }

    /// set encodeing policy: auto, fast or size
    pub fn set_encode_policy(&mut self, encode_policy: EncodePolicy) {
        self.encode_policy = encode_policy;
    }
}
