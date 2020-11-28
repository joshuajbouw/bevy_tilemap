//! # Bevy Tilemap Types
//!
//! All the extra or helpful types that are not supported by Bevy or Glam are
//! all contained here.

#![no_implicit_prelude]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
// rustc
#![deny(dead_code, missing_docs, unused_imports)]

extern crate bevy;
#[cfg(feature = "serde")]
extern crate serde;
extern crate std;

/// Dimension helpers with encoding and decoding to and from indexes.
pub mod dimension;
/// Points used for helping with coordinates.
pub mod point;
pub mod prelude;

/// A custom prelude around all the types we need from `std`, `bevy`, and `serde`.
mod lib {
    use ::bevy::math as bevy_math;

    pub(crate) use self::bevy_math::{Vec2, Vec3};

    #[cfg(feature = "serde")]
    pub(crate) use ::serde::{Deserialize, Serialize};

    pub(crate) use ::std::{
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
    pub(crate) use ::std::write;

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub(crate) use ::std::println;
}
