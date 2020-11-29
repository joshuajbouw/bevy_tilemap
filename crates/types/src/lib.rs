//! # Bevy Tilemap Types
//!
//! All the extra or helpful types that are not supported by Bevy or Glam are
//! all contained here.

#![no_implicit_prelude]
// rustc
#![deny(dead_code, unused_imports)]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(clippy::print_stdout, clippy::unwrap_in_result)]
#![deny(
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::decimal_literal_representation,
    clippy::else_if_without_else,
    // clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::panic_in_result_fn,
    clippy::cast_lossless,
    clippy::redundant_pub_crate,
    // clippy::missing_inline_in_public_items,
)]

/// Dimension helpers with encoding and decoding to and from indexes.
pub mod dimension;
/// Points used for helping with coordinates.
pub mod point;
pub mod prelude;

/// A custom prelude around all the types we need from `std`, `bevy`, and `serde`.
mod lib {
    extern crate bevy;
    #[cfg(feature = "serde")]
    extern crate serde;
    extern crate std;

    use bevy::math as bevy_math;

    pub use self::bevy_math::{Vec2, Vec3};

    #[cfg(feature = "serde")]
    pub use serde::{Deserialize, Serialize};

    pub use std::{
        boxed::Box,
        clone::Clone,
        cmp::Ord,
        convert::{From, Into},
        default::Default,
        error::Error,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
        result::Result::{self, *},
    };

    // MacrosD
    pub use std::write;

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub use std::println;
}
