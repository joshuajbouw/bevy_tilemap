//! # Bevy Tilemap Types
//!
//! All the extra or helpful types that are not supported by Bevy or Glam are
//! all contained here.

#![doc(html_root_url = "https://docs.rs/bevy_tilemap_types/0.4.0")]
// rustc / rustdoc
// This won't build on stable releases until it is stable.
//#![warn(rustdoc::private_doc_tests)]
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

pub mod dimension;
pub mod point;
pub mod prelude;

/// A custom prelude around all the types we need from `std`, `bevy`, and `serde`.
mod lib {
    pub(crate) use bevy::{
        ecs::{component::Component, reflect::ReflectComponent},
        math::{Vec2, Vec3},
        reflect::{Reflect, ReflectDeserialize},
        render::texture::Extent3d,
    };

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
        result::Result::{self, *},
    };

    // Macros
    pub(crate) use std::write;

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub(crate) use std::println;
}
