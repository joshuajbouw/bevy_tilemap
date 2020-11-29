//! The Bevy Tilemap types prelude.
//!
//! # Prelude contents
//!
//! The current version of this prelude (version 0) is located in
//! [`bevy_tilemap_types::prelude::v0`], and re-exports the following.
//!
//! * [`bevy_tilemap_types::dimension`]::{[`Dimension2`], [`Dimension3`]}
//! common methods and helpers for dealing with dimensions.
//! * [`bevy_tilemap_types::point`]::{[`Point2`], [`Point3`]} common
//! methods and helpers for dealing with points of the 2nd and 3rd dimension.
//!
//! [`bevy_tilemap_types::prelude::v0`]: crate::prelude::v0
//! [`bevy_tilemap_types::dimension`]: crate::dimension
//! [`bevy_tilemap_types::point`]: crate::point
//! [`Dimension2`]: crate::dimension::Dimension2
//! [`Dimension3`]: crate::dimension::Dimension3
//! [`Point2`]: crate::point::Point2
//! [`Point3`]: crate::point::Point3

/// The v0 prelude version of Bevy Tilemap Types.
pub mod v0 {
    pub use crate::{
        dimension::{Dimension2, Dimension3},
        point::{Point2, Point3},
    };
}
