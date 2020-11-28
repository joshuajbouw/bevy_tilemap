//! The Bevy Tilemap prelude.
//!
//! # Prelude contents
//!
//! The current version of this prelude (version 0) is located in
//! [`bevy_tilemap_2d::prelude::v0`], and re-exports the following.
//!
//! * [`bevy_tilemap_2d::chunk`]::[`LayerKind`], the only public part
//! of `chunk` module is the kind of layer you need to specify to create.
//! * [`bevy_tilemap_2d::entity`]::[`TilemapComponents`], the components
//! for spawning with a Tilemap.
//! * [`bevy_tilemap_2d::tilemap`]::{[`Tilemap`], [`TilemapBuilder`]},
//! the core object that is used for virtually everything in this library.
//! * [`bevy_tilemap_2d::tile`]::[`Tile`], a sprite tile which
//! holds minimal amount of data for small data sizes.
//! * [`bevy_tilemap_2d`]::[`Tilemap2DPlugin`], the main plugin with
//! a collection of systems, components and assets to be used in a Bevy app.
//!
//! [`bevy_tilemap_2d::prelude::v0`]: crate::prelude::v0
//! [`bevy_tilemap_2d::chunk`]: crate::chunk
//! [`bevy_tilemap_2d::entity`]: crate::entity
//! [`bevy_tilemap_2d::tilemap`]: crate::tilemap
//! [`bevy_tilemap_2d::tile`]: crate::tile
//! [`bevy_tilemap_2d`]: crate

/// Version 0 prelude.
pub mod v0 {
    #[cfg(feature = "types")]
    pub use crate::bevy_tilemap_types::prelude::v0::*;
    pub use crate::{
        bevy_tilemap_spritesheet::prelude::v0::*,
        chunk::LayerKind,
        entity::TilemapComponents,
        tile::Tile,
        tilemap::{Tilemap, TilemapBuilder},
        Tilemap2DPlugin,
    };
}

pub use v0::*;
