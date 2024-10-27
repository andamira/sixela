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

mod dither;
mod error;
mod other;
mod tosixel;

pub mod output;
pub mod pixelformat;
pub mod quant;

/// All items are flat re-exported here. <br/><hr>
pub mod all {
    #[doc(inline)]
    pub use super::{dither::*, error::*, other::*, output::*, pixelformat::*, quant::*};
}
#[doc(hidden)]
pub use all::*;

// limitations
const SIXEL_OUTPUT_PACKET_SIZE: usize = 16_384;
const SIXEL_PALETTE_MIN: usize = 2;
const SIXEL_PALETTE_MAX: usize = 256;
const SIXEL_USE_DEPRECATED_SYMBOLS: usize = 1;
const SIXEL_ALLOCATE_BYTES_MAX: usize = 10_248 * 1_024 * 128; /* up to 128M */
const SIXEL_WIDTH_LIMIT: usize = 1_000_000;
const SIXEL_HEIGHT_LIMIT: usize = 1_000_000;

// loader settings
const SIXEL_DEFALUT_GIF_DELAY: usize = 1;
