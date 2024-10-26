use std::vec;

use crate::{
    pixelformat::sixel_helper_normalize_pixelformat,
    quant::{sixel_quant_apply_palette, sixel_quant_make_palette},
    BuiltinDither, DiffusionMethod, MethodForLargest, MethodForRep, PixelFormat, Quality,
    SixelError, SixelResult, SIXEL_PALETTE_MAX,
};

pub struct sixel_dither {
    pub palette: Vec<u8>,             /* palette definition */
    pub cachetable: Option<Vec<u16>>, /* cache table */
    pub reqcolors: i32,               /* requested colors */
    pub ncolors: i32,                 /* active colors */
    pub origcolors: i32,              /* original colors */
    pub optimized: bool,              /* pixel is 15bpp compressable */
    pub optimize_palette: bool,       /* minimize palette size */
    pub complexion: i32,              /* for complexion correction */
    pub bodyonly: bool,               /* do not output palette section if true */
    pub method_for_largest: MethodForLargest, /* method for finding the largest dimention
                                      for splitting */
    pub method_for_rep: MethodForRep, /* method for choosing a color from the box */
    pub method_for_diffuse: DiffusionMethod, /* method for diffusing */
    pub quality_mode: Quality,        /* quality of histogram */
    pub keycolor: i32,                /* background color */
    pub pixelformat: PixelFormat,     /* pixelformat for internal processing */
}

const pal_mono_dark: [u8; 6] = [0x00, 0x00, 0x00, 0xff, 0xff, 0xff];

const pal_mono_light: [u8; 6] = [0xff, 0xff, 0xff, 0x00, 0x00, 0x00];

const pal_gray_1bit: [u8; 6] = [0x00, 0x00, 0x00, 0xff, 0xff, 0xff];

const pal_gray_2bit: [u8; 12] =
    [0x00, 0x00, 0x00, 0x55, 0x55, 0x55, 0xaa, 0xaa, 0xaa, 0xff, 0xff, 0xff];

const pal_gray_4bit: [u8; 48] = [
    0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x22, 0x22, 0x22, 0x33, 0x33, 0x33, 0x44, 0x44, 0x44, 0x55,
    0x55, 0x55, 0x66, 0x66, 0x66, 0x77, 0x77, 0x77, 0x88, 0x88, 0x88, 0x99, 0x99, 0x99, 0xaa, 0xaa,
    0xaa, 0xbb, 0xbb, 0xbb, 0xcc, 0xcc, 0xcc, 0xdd, 0xdd, 0xdd, 0xee, 0xee, 0xee, 0xff, 0xff, 0xff,
];

const pal_gray_8bit: [u8; 768] = [
    0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x03, 0x03, 0x03, 0x04, 0x04, 0x04, 0x05,
    0x05, 0x05, 0x06, 0x06, 0x06, 0x07, 0x07, 0x07, 0x08, 0x08, 0x08, 0x09, 0x09, 0x09, 0x0a, 0x0a,
    0x0a, 0x0b, 0x0b, 0x0b, 0x0c, 0x0c, 0x0c, 0x0d, 0x0d, 0x0d, 0x0e, 0x0e, 0x0e, 0x0f, 0x0f, 0x0f,
    0x10, 0x10, 0x10, 0x11, 0x11, 0x11, 0x12, 0x12, 0x12, 0x13, 0x13, 0x13, 0x14, 0x14, 0x14, 0x15,
    0x15, 0x15, 0x16, 0x16, 0x16, 0x17, 0x17, 0x17, 0x18, 0x18, 0x18, 0x19, 0x19, 0x19, 0x1a, 0x1a,
    0x1a, 0x1b, 0x1b, 0x1b, 0x1c, 0x1c, 0x1c, 0x1d, 0x1d, 0x1d, 0x1e, 0x1e, 0x1e, 0x1f, 0x1f, 0x1f,
    0x20, 0x20, 0x20, 0x21, 0x21, 0x21, 0x22, 0x22, 0x22, 0x23, 0x23, 0x23, 0x24, 0x24, 0x24, 0x25,
    0x25, 0x25, 0x26, 0x26, 0x26, 0x27, 0x27, 0x27, 0x28, 0x28, 0x28, 0x29, 0x29, 0x29, 0x2a, 0x2a,
    0x2a, 0x2b, 0x2b, 0x2b, 0x2c, 0x2c, 0x2c, 0x2d, 0x2d, 0x2d, 0x2e, 0x2e, 0x2e, 0x2f, 0x2f, 0x2f,
    0x30, 0x30, 0x30, 0x31, 0x31, 0x31, 0x32, 0x32, 0x32, 0x33, 0x33, 0x33, 0x34, 0x34, 0x34, 0x35,
    0x35, 0x35, 0x36, 0x36, 0x36, 0x37, 0x37, 0x37, 0x38, 0x38, 0x38, 0x39, 0x39, 0x39, 0x3a, 0x3a,
    0x3a, 0x3b, 0x3b, 0x3b, 0x3c, 0x3c, 0x3c, 0x3d, 0x3d, 0x3d, 0x3e, 0x3e, 0x3e, 0x3f, 0x3f, 0x3f,
    0x40, 0x40, 0x40, 0x41, 0x41, 0x41, 0x42, 0x42, 0x42, 0x43, 0x43, 0x43, 0x44, 0x44, 0x44, 0x45,
    0x45, 0x45, 0x46, 0x46, 0x46, 0x47, 0x47, 0x47, 0x48, 0x48, 0x48, 0x49, 0x49, 0x49, 0x4a, 0x4a,
    0x4a, 0x4b, 0x4b, 0x4b, 0x4c, 0x4c, 0x4c, 0x4d, 0x4d, 0x4d, 0x4e, 0x4e, 0x4e, 0x4f, 0x4f, 0x4f,
    0x50, 0x50, 0x50, 0x51, 0x51, 0x51, 0x52, 0x52, 0x52, 0x53, 0x53, 0x53, 0x54, 0x54, 0x54, 0x55,
    0x55, 0x55, 0x56, 0x56, 0x56, 0x57, 0x57, 0x57, 0x58, 0x58, 0x58, 0x59, 0x59, 0x59, 0x5a, 0x5a,
    0x5a, 0x5b, 0x5b, 0x5b, 0x5c, 0x5c, 0x5c, 0x5d, 0x5d, 0x5d, 0x5e, 0x5e, 0x5e, 0x5f, 0x5f, 0x5f,
    0x60, 0x60, 0x60, 0x61, 0x61, 0x61, 0x62, 0x62, 0x62, 0x63, 0x63, 0x63, 0x64, 0x64, 0x64, 0x65,
    0x65, 0x65, 0x66, 0x66, 0x66, 0x67, 0x67, 0x67, 0x68, 0x68, 0x68, 0x69, 0x69, 0x69, 0x6a, 0x6a,
    0x6a, 0x6b, 0x6b, 0x6b, 0x6c, 0x6c, 0x6c, 0x6d, 0x6d, 0x6d, 0x6e, 0x6e, 0x6e, 0x6f, 0x6f, 0x6f,
    0x70, 0x70, 0x70, 0x71, 0x71, 0x71, 0x72, 0x72, 0x72, 0x73, 0x73, 0x73, 0x74, 0x74, 0x74, 0x75,
    0x75, 0x75, 0x76, 0x76, 0x76, 0x77, 0x77, 0x77, 0x78, 0x78, 0x78, 0x79, 0x79, 0x79, 0x7a, 0x7a,
    0x7a, 0x7b, 0x7b, 0x7b, 0x7c, 0x7c, 0x7c, 0x7d, 0x7d, 0x7d, 0x7e, 0x7e, 0x7e, 0x7f, 0x7f, 0x7f,
    0x80, 0x80, 0x80, 0x81, 0x81, 0x81, 0x82, 0x82, 0x82, 0x83, 0x83, 0x83, 0x84, 0x84, 0x84, 0x85,
    0x85, 0x85, 0x86, 0x86, 0x86, 0x87, 0x87, 0x87, 0x88, 0x88, 0x88, 0x89, 0x89, 0x89, 0x8a, 0x8a,
    0x8a, 0x8b, 0x8b, 0x8b, 0x8c, 0x8c, 0x8c, 0x8d, 0x8d, 0x8d, 0x8e, 0x8e, 0x8e, 0x8f, 0x8f, 0x8f,
    0x90, 0x90, 0x90, 0x91, 0x91, 0x91, 0x92, 0x92, 0x92, 0x93, 0x93, 0x93, 0x94, 0x94, 0x94, 0x95,
    0x95, 0x95, 0x96, 0x96, 0x96, 0x97, 0x97, 0x97, 0x98, 0x98, 0x98, 0x99, 0x99, 0x99, 0x9a, 0x9a,
    0x9a, 0x9b, 0x9b, 0x9b, 0x9c, 0x9c, 0x9c, 0x9d, 0x9d, 0x9d, 0x9e, 0x9e, 0x9e, 0x9f, 0x9f, 0x9f,
    0xa0, 0xa0, 0xa0, 0xa1, 0xa1, 0xa1, 0xa2, 0xa2, 0xa2, 0xa3, 0xa3, 0xa3, 0xa4, 0xa4, 0xa4, 0xa5,
    0xa5, 0xa5, 0xa6, 0xa6, 0xa6, 0xa7, 0xa7, 0xa7, 0xa8, 0xa8, 0xa8, 0xa9, 0xa9, 0xa9, 0xaa, 0xaa,
    0xaa, 0xab, 0xab, 0xab, 0xac, 0xac, 0xac, 0xad, 0xad, 0xad, 0xae, 0xae, 0xae, 0xaf, 0xaf, 0xaf,
    0xb0, 0xb0, 0xb0, 0xb1, 0xb1, 0xb1, 0xb2, 0xb2, 0xb2, 0xb3, 0xb3, 0xb3, 0xb4, 0xb4, 0xb4, 0xb5,
    0xb5, 0xb5, 0xb6, 0xb6, 0xb6, 0xb7, 0xb7, 0xb7, 0xb8, 0xb8, 0xb8, 0xb9, 0xb9, 0xb9, 0xba, 0xba,
    0xba, 0xbb, 0xbb, 0xbb, 0xbc, 0xbc, 0xbc, 0xbd, 0xbd, 0xbd, 0xbe, 0xbe, 0xbe, 0xbf, 0xbf, 0xbf,
    0xc0, 0xc0, 0xc0, 0xc1, 0xc1, 0xc1, 0xc2, 0xc2, 0xc2, 0xc3, 0xc3, 0xc3, 0xc4, 0xc4, 0xc4, 0xc5,
    0xc5, 0xc5, 0xc6, 0xc6, 0xc6, 0xc7, 0xc7, 0xc7, 0xc8, 0xc8, 0xc8, 0xc9, 0xc9, 0xc9, 0xca, 0xca,
    0xca, 0xcb, 0xcb, 0xcb, 0xcc, 0xcc, 0xcc, 0xcd, 0xcd, 0xcd, 0xce, 0xce, 0xce, 0xcf, 0xcf, 0xcf,
    0xd0, 0xd0, 0xd0, 0xd1, 0xd1, 0xd1, 0xd2, 0xd2, 0xd2, 0xd3, 0xd3, 0xd3, 0xd4, 0xd4, 0xd4, 0xd5,
    0xd5, 0xd5, 0xd6, 0xd6, 0xd6, 0xd7, 0xd7, 0xd7, 0xd8, 0xd8, 0xd8, 0xd9, 0xd9, 0xd9, 0xda, 0xda,
    0xda, 0xdb, 0xdb, 0xdb, 0xdc, 0xdc, 0xdc, 0xdd, 0xdd, 0xdd, 0xde, 0xde, 0xde, 0xdf, 0xdf, 0xdf,
    0xe0, 0xe0, 0xe0, 0xe1, 0xe1, 0xe1, 0xe2, 0xe2, 0xe2, 0xe3, 0xe3, 0xe3, 0xe4, 0xe4, 0xe4, 0xe5,
    0xe5, 0xe5, 0xe6, 0xe6, 0xe6, 0xe7, 0xe7, 0xe7, 0xe8, 0xe8, 0xe8, 0xe9, 0xe9, 0xe9, 0xea, 0xea,
    0xea, 0xeb, 0xeb, 0xeb, 0xec, 0xec, 0xec, 0xed, 0xed, 0xed, 0xee, 0xee, 0xee, 0xef, 0xef, 0xef,
    0xf0, 0xf0, 0xf0, 0xf1, 0xf1, 0xf1, 0xf2, 0xf2, 0xf2, 0xf3, 0xf3, 0xf3, 0xf4, 0xf4, 0xf4, 0xf5,
    0xf5, 0xf5, 0xf6, 0xf6, 0xf6, 0xf7, 0xf7, 0xf7, 0xf8, 0xf8, 0xf8, 0xf9, 0xf9, 0xf9, 0xfa, 0xfa,
    0xfa, 0xfb, 0xfb, 0xfb, 0xfc, 0xfc, 0xfc, 0xfd, 0xfd, 0xfd, 0xfe, 0xfe, 0xfe, 0xff, 0xff, 0xff,
];

const pal_xterm256: [u8; 768] = [
    0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x80, 0x00, 0x80, 0x80, 0x00, 0x00, 0x00, 0x80, 0x80,
    0x00, 0x80, 0x00, 0x80, 0x80, 0xc0, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0xff, 0x00, 0x00, 0x00, 0xff,
    0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x5f, 0x00, 0x00, 0x87, 0x00, 0x00, 0xaf, 0x00, 0x00, 0xd7, 0x00,
    0x00, 0xff, 0x00, 0x5f, 0x00, 0x00, 0x5f, 0x5f, 0x00, 0x5f, 0x87, 0x00, 0x5f, 0xaf, 0x00, 0x5f,
    0xd7, 0x00, 0x5f, 0xff, 0x00, 0x87, 0x00, 0x00, 0x87, 0x5f, 0x00, 0x87, 0x87, 0x00, 0x87, 0xaf,
    0x00, 0x87, 0xd7, 0x00, 0x87, 0xff, 0x00, 0xaf, 0x00, 0x00, 0xaf, 0x5f, 0x00, 0xaf, 0x87, 0x00,
    0xaf, 0xaf, 0x00, 0xaf, 0xd7, 0x00, 0xaf, 0xff, 0x00, 0xd7, 0x00, 0x00, 0xd7, 0x5f, 0x00, 0xd7,
    0x87, 0x00, 0xd7, 0xaf, 0x00, 0xd7, 0xd7, 0x00, 0xd7, 0xff, 0x00, 0xff, 0x00, 0x00, 0xff, 0x5f,
    0x00, 0xff, 0x87, 0x00, 0xff, 0xaf, 0x00, 0xff, 0xd7, 0x00, 0xff, 0xff, 0x5f, 0x00, 0x00, 0x5f,
    0x00, 0x5f, 0x5f, 0x00, 0x87, 0x5f, 0x00, 0xaf, 0x5f, 0x00, 0xd7, 0x5f, 0x00, 0xff, 0x5f, 0x5f,
    0x00, 0x5f, 0x5f, 0x5f, 0x5f, 0x5f, 0x87, 0x5f, 0x5f, 0xaf, 0x5f, 0x5f, 0xd7, 0x5f, 0x5f, 0xff,
    0x5f, 0x87, 0x00, 0x5f, 0x87, 0x5f, 0x5f, 0x87, 0x87, 0x5f, 0x87, 0xaf, 0x5f, 0x87, 0xd7, 0x5f,
    0x87, 0xff, 0x5f, 0xaf, 0x00, 0x5f, 0xaf, 0x5f, 0x5f, 0xaf, 0x87, 0x5f, 0xaf, 0xaf, 0x5f, 0xaf,
    0xd7, 0x5f, 0xaf, 0xff, 0x5f, 0xd7, 0x00, 0x5f, 0xd7, 0x5f, 0x5f, 0xd7, 0x87, 0x5f, 0xd7, 0xaf,
    0x5f, 0xd7, 0xd7, 0x5f, 0xd7, 0xff, 0x5f, 0xff, 0x00, 0x5f, 0xff, 0x5f, 0x5f, 0xff, 0x87, 0x5f,
    0xff, 0xaf, 0x5f, 0xff, 0xd7, 0x5f, 0xff, 0xff, 0x87, 0x00, 0x00, 0x87, 0x00, 0x5f, 0x87, 0x00,
    0x87, 0x87, 0x00, 0xaf, 0x87, 0x00, 0xd7, 0x87, 0x00, 0xff, 0x87, 0x5f, 0x00, 0x87, 0x5f, 0x5f,
    0x87, 0x5f, 0x87, 0x87, 0x5f, 0xaf, 0x87, 0x5f, 0xd7, 0x87, 0x5f, 0xff, 0x87, 0x87, 0x00, 0x87,
    0x87, 0x5f, 0x87, 0x87, 0x87, 0x87, 0x87, 0xaf, 0x87, 0x87, 0xd7, 0x87, 0x87, 0xff, 0x87, 0xaf,
    0x00, 0x87, 0xaf, 0x5f, 0x87, 0xaf, 0x87, 0x87, 0xaf, 0xaf, 0x87, 0xaf, 0xd7, 0x87, 0xaf, 0xff,
    0x87, 0xd7, 0x00, 0x87, 0xd7, 0x5f, 0x87, 0xd7, 0x87, 0x87, 0xd7, 0xaf, 0x87, 0xd7, 0xd7, 0x87,
    0xd7, 0xff, 0x87, 0xff, 0x00, 0x87, 0xff, 0x5f, 0x87, 0xff, 0x87, 0x87, 0xff, 0xaf, 0x87, 0xff,
    0xd7, 0x87, 0xff, 0xff, 0xaf, 0x00, 0x00, 0xaf, 0x00, 0x5f, 0xaf, 0x00, 0x87, 0xaf, 0x00, 0xaf,
    0xaf, 0x00, 0xd7, 0xaf, 0x00, 0xff, 0xaf, 0x5f, 0x00, 0xaf, 0x5f, 0x5f, 0xaf, 0x5f, 0x87, 0xaf,
    0x5f, 0xaf, 0xaf, 0x5f, 0xd7, 0xaf, 0x5f, 0xff, 0xaf, 0x87, 0x00, 0xaf, 0x87, 0x5f, 0xaf, 0x87,
    0x87, 0xaf, 0x87, 0xaf, 0xaf, 0x87, 0xd7, 0xaf, 0x87, 0xff, 0xaf, 0xaf, 0x00, 0xaf, 0xaf, 0x5f,
    0xaf, 0xaf, 0x87, 0xaf, 0xaf, 0xaf, 0xaf, 0xaf, 0xd7, 0xaf, 0xaf, 0xff, 0xaf, 0xd7, 0x00, 0xaf,
    0xd7, 0x5f, 0xaf, 0xd7, 0x87, 0xaf, 0xd7, 0xaf, 0xaf, 0xd7, 0xd7, 0xaf, 0xd7, 0xff, 0xaf, 0xff,
    0x00, 0xaf, 0xff, 0x5f, 0xaf, 0xff, 0x87, 0xaf, 0xff, 0xaf, 0xaf, 0xff, 0xd7, 0xaf, 0xff, 0xff,
    0xd7, 0x00, 0x00, 0xd7, 0x00, 0x5f, 0xd7, 0x00, 0x87, 0xd7, 0x00, 0xaf, 0xd7, 0x00, 0xd7, 0xd7,
    0x00, 0xff, 0xd7, 0x5f, 0x00, 0xd7, 0x5f, 0x5f, 0xd7, 0x5f, 0x87, 0xd7, 0x5f, 0xaf, 0xd7, 0x5f,
    0xd7, 0xd7, 0x5f, 0xff, 0xd7, 0x87, 0x00, 0xd7, 0x87, 0x5f, 0xd7, 0x87, 0x87, 0xd7, 0x87, 0xaf,
    0xd7, 0x87, 0xd7, 0xd7, 0x87, 0xff, 0xd7, 0xaf, 0x00, 0xd7, 0xaf, 0x5f, 0xd7, 0xaf, 0x87, 0xd7,
    0xaf, 0xaf, 0xd7, 0xaf, 0xd7, 0xd7, 0xaf, 0xff, 0xd7, 0xd7, 0x00, 0xd7, 0xd7, 0x5f, 0xd7, 0xd7,
    0x87, 0xd7, 0xd7, 0xaf, 0xd7, 0xd7, 0xd7, 0xd7, 0xd7, 0xff, 0xd7, 0xff, 0x00, 0xd7, 0xff, 0x5f,
    0xd7, 0xff, 0x87, 0xd7, 0xff, 0xaf, 0xd7, 0xff, 0xd7, 0xd7, 0xff, 0xff, 0xff, 0x00, 0x00, 0xff,
    0x00, 0x5f, 0xff, 0x00, 0x87, 0xff, 0x00, 0xaf, 0xff, 0x00, 0xd7, 0xff, 0x00, 0xff, 0xff, 0x5f,
    0x00, 0xff, 0x5f, 0x5f, 0xff, 0x5f, 0x87, 0xff, 0x5f, 0xaf, 0xff, 0x5f, 0xd7, 0xff, 0x5f, 0xff,
    0xff, 0x87, 0x00, 0xff, 0x87, 0x5f, 0xff, 0x87, 0x87, 0xff, 0x87, 0xaf, 0xff, 0x87, 0xd7, 0xff,
    0x87, 0xff, 0xff, 0xaf, 0x00, 0xff, 0xaf, 0x5f, 0xff, 0xaf, 0x87, 0xff, 0xaf, 0xaf, 0xff, 0xaf,
    0xd7, 0xff, 0xaf, 0xff, 0xff, 0xd7, 0x00, 0xff, 0xd7, 0x5f, 0xff, 0xd7, 0x87, 0xff, 0xd7, 0xaf,
    0xff, 0xd7, 0xd7, 0xff, 0xd7, 0xff, 0xff, 0xff, 0x00, 0xff, 0xff, 0x5f, 0xff, 0xff, 0x87, 0xff,
    0xff, 0xaf, 0xff, 0xff, 0xd7, 0xff, 0xff, 0xff, 0x08, 0x08, 0x08, 0x12, 0x12, 0x12, 0x1c, 0x1c,
    0x1c, 0x26, 0x26, 0x26, 0x30, 0x30, 0x30, 0x3a, 0x3a, 0x3a, 0x44, 0x44, 0x44, 0x4e, 0x4e, 0x4e,
    0x58, 0x58, 0x58, 0x62, 0x62, 0x62, 0x6c, 0x6c, 0x6c, 0x76, 0x76, 0x76, 0x80, 0x80, 0x80, 0x8a,
    0x8a, 0x8a, 0x94, 0x94, 0x94, 0x9e, 0x9e, 0x9e, 0xa8, 0xa8, 0xa8, 0xb2, 0xb2, 0xb2, 0xbc, 0xbc,
    0xbc, 0xc6, 0xc6, 0xc6, 0xd0, 0xd0, 0xd0, 0xda, 0xda, 0xda, 0xe4, 0xe4, 0xe4, 0xee, 0xee, 0xee,
];

/*
 * VT340 undocumented behavior regarding the color palette reported
 * by Vertis Sidus(@vrtsds):
 *     it loads the first fifteen colors as 1 through 15, and loads the
 *     sixteenth color as 0.
 */
const pal_vt340_mono: [u8; 48] = [
    0x21, 0x21, 0x21, // 1   Gray-2
    0x42, 0x42, 0x42, // 2   Gray-4
    0x66, 0x66, 0x66, // 3   Gray-6
    0x0F, 0x0F, 0x0F, // 4   Gray-1
    0x33, 0x33, 0x33, // 5   Gray-3
    0x54, 0x54, 0x54, // 6   Gray-5
    0x75, 0x75, 0x75, // 7   White 7
    0x00, 0x00, 0x00, // 8   Black 0
    0x21, 0x21, 0x21, // 9   Gray-2
    0x42, 0x42, 0x42, // 10  Gray-4
    0x66, 0x66, 0x66, // 11  Gray-6
    0x0F, 0x0F, 0x0F, // 12  Gray-1
    0x33, 0x33, 0x33, // 13  Gray-3
    0x54, 0x54, 0x54, // 14  Gray-5
    0x75, 0x75, 0x75, // 15  White 7
    0x00, 0x00, 0x00, // 0   Black
];

const pal_vt340_color: [u8; 48] = [
    0x21, 0x21, 0x21, // 1   Gray-2
    0x42, 0x42, 0x42, // 2   Gray-4
    0x66, 0x66, 0x66, // 3   Gray-6
    0x0F, 0x0F, 0x0F, // 4   Gray-1
    0x33, 0x33, 0x33, // 5   Gray-3
    0x54, 0x54, 0x54, // 6   Gray-5
    0x75, 0x75, 0x75, // 7   White 7
    0x00, 0x00, 0x00, // 8   Black 0
    0x21, 0x21, 0x21, // 9   Gray-2
    0x42, 0x42, 0x42, // 10  Gray-4
    0x66, 0x66, 0x66, // 11  Gray-6
    0x0F, 0x0F, 0x0F, // 12  Gray-1
    0x33, 0x33, 0x33, // 13  Gray-3
    0x54, 0x54, 0x54, // 14  Gray-5
    0x75, 0x75, 0x75, // 15  White 7
    0x00, 0x00, 0x00, // 0   Black
];

impl sixel_dither {
    /// create dither context object
    ///     /* required colors */
    pub fn new(mut ncolors: i32) -> SixelResult<Self> {
        let quality_mode = if ncolors < 0 {
            ncolors = SIXEL_PALETTE_MAX as i32;
            Quality::HIGHCOLOR
        } else {
            if ncolors > SIXEL_PALETTE_MAX as i32 {
                return Err(Box::new(SixelError::BadInput));
            }
            if ncolors < 1 {
                return Err(Box::new(SixelError::BadInput));
                //                sixel_helper_set_additional_message(
                //                  "sixel_dither_new: palette colors must be more than 0");
            }
            Quality::LOW
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
            method_for_diffuse: DiffusionMethod::FS,
            quality_mode,
            pixelformat: PixelFormat::RGB888,
        })
    }

    pub fn sixel_dither_get(builtin_dither: BuiltinDither) -> SixelResult<Self> {
        let (ncolors, palette, keycolor) = match builtin_dither {
            BuiltinDither::MonoDark => (2, pal_mono_dark.to_vec(), 0),
            BuiltinDither::MonoLight => (2, pal_mono_light.to_vec(), 0),
            BuiltinDither::XTerm16 => (16, pal_xterm256.to_vec(), -1),
            BuiltinDither::XTerm256 => (256, pal_xterm256.to_vec(), -1),
            BuiltinDither::VT340Mono => (16, pal_vt340_mono.to_vec(), -1),
            BuiltinDither::VT340Color => (16, pal_vt340_color.to_vec(), -1),
            BuiltinDither::G1 => (2, pal_gray_1bit.to_vec(), -1),
            BuiltinDither::G2 => (4, pal_gray_2bit.to_vec(), -1),
            BuiltinDither::G4 => (16, pal_gray_4bit.to_vec(), -1),
            BuiltinDither::G8 => (256, pal_gray_8bit.to_vec(), -1),
        };

        let mut result = sixel_dither::new(ncolors)?;
        result.palette = palette;
        result.keycolor = keycolor;
        result.optimized = true;
        result.optimize_palette = false;
        Ok(result)
    }

    pub fn set_method_for_largest(&mut self, method_for_largest: MethodForLargest) {
        self.method_for_largest = if matches!(method_for_largest, MethodForLargest::Auto) {
            MethodForLargest::Norm
        } else {
            method_for_largest
        };
    }

    pub fn set_method_for_rep(&mut self, method_for_rep: MethodForRep) {
        self.method_for_rep = if matches!(method_for_rep, MethodForRep::Auto) {
            MethodForRep::CenterBox
        } else {
            method_for_rep
        };
    }

    pub fn set_quality_mode(&mut self, quality_mode: Quality) {
        self.quality_mode = if matches!(quality_mode, Quality::AUTO) {
            if self.ncolors <= 8 {
                Quality::HIGH
            } else {
                Quality::LOW
            }
        } else {
            quality_mode
        };
    }

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
            self.method_for_diffuse = DiffusionMethod::None;
        }
        Ok(())
    }

    /// set diffusion type, choose from enum methodForDiffuse
    pub fn set_diffusion_type(&mut self, method_for_diffuse: DiffusionMethod) {
        self.method_for_diffuse = if matches!(method_for_diffuse, DiffusionMethod::Auto) {
            if self.ncolors > 16 {
                DiffusionMethod::FS
            } else {
                DiffusionMethod::Atkinson
            }
        } else {
            method_for_diffuse
        };
    }

    /// get number of palette colors
    pub fn get_num_of_palette_colors(&self) -> i32 {
        self.ncolors
    }

    /// get number of histogram colors
    pub fn get_num_of_histogram_colors(&self) -> i32 {
        self.origcolors
    }

    /// get palette
    pub fn get_palette(&self) -> &[u8] {
        &self.palette
    }

    /// set palette
    pub fn set_palette(&mut self, palette: Vec<u8>) {
        self.palette = palette;
    }

    /// set the factor of complexion color correcting
    //  /* complexion score (>= 1) */
    pub fn set_complexion_score(&mut self, score: i32) {
        self.complexion = score;
    }

    /* set whether omitting palette difinition */
    pub fn set_body_only(&mut self, bodyonly: bool)
    /* 0: output palette section
    1: do not output palette section  */
    {
        self.bodyonly = bodyonly;
    }

    /* set whether optimize palette size */
    pub fn set_optimize_palette(&mut self, do_op: bool)
    /* 0: optimize palette size
    1: don't optimize palette size */
    {
        self.optimize_palette = do_op;
    }

    /* set pixelformat */
    pub fn set_pixelformat(&mut self, pixelformat: PixelFormat) /* one of enum pixelFormat */
    {
        self.pixelformat = pixelformat;
    }

    /* set transparent */
    pub fn set_transparent(&mut self, transparent: i32) /* transparent color index */
    {
        self.keycolor = transparent;
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
        if matches!(self.quality_mode, Quality::FULL) {
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
