//! # Bevy Tilemap Types
//!
//! All the extra or helpful types that are not supported by Bevy or Glam are
//! all contained here.

#![doc(html_root_url = "https://docs.rs/bevy_tilemap_types/0.1.1")]
// This was broken even further and no longer will work at all with the previous
// workaround. There is a fix, might be sometime for it to be included though.
// Even then, it is just a warning. For now, including it per module seems to
// fix it.
// See: https://github.com/rust-lang/rust/pull/80372
// #![no_implicit_prelude]

// rustc / rustdoc
#![warn(missing_docs, private_doc_tests)]
#![deny(dead_code, unused_imports)]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(
    clippy::cast_lossless,
    clippy::decimal_literal_representation,
    clippy::else_if_without_else,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::panic_in_result_fn,
    clippy::print_stdout,
    clippy::unwrap_in_result
)]

#[no_implicit_prelude]
pub mod dimension;
#[no_implicit_prelude]
pub mod point;
#[no_implicit_prelude]
pub mod prelude;

/// A custom prelude around all the types we need from `std`, `bevy`, and `serde`.
#[no_implicit_prelude]
mod lib {
    extern crate bevy_math;
    extern crate bevy_render;
    #[cfg(feature = "serde")]
    extern crate serde;
    extern crate std;

    pub(crate) use self::{
        bevy_math::{Vec2, Vec3},
        bevy_render::texture::Extent3d,
    };

    #[cfg(feature = "serde")]
    pub(crate) use serde::{Deserialize, Serialize};

    pub(crate) use std::{
        boxed::Box,
        clone::Clone,
        cmp::Ord,
        convert::{From, Into},
        default::Default,
        error::Error,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
        option::Option::{self, *},
        result::Result::{self, *},
    };

    // Macros
    pub(crate) use std::write;

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub(crate) use std::println;
}
