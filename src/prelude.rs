//! The Bevy Tilemap Prelude.
//!
//! Since there is not a lot public declarations and much of the library is
//! mostly private API, everything that you would likely use is included in
//! here.
//!
//! While there is acknowledgement that there is some useful private API that
//! will be useful publicly, this should happen in another crate as it is out
//! of scope.
//!
//! # Prelude contents
//!
//! The current version of this prelude (version 0.2) is located in
//! [`bevy_tilemap::prelude::v0_2`], and re-exports the following.
//!
//! * [`bevy_tilemap::chunk`]::[`LayerKind`], the only public part
//! of `chunk` module is the kind of layer you need to specify to create.
//! * [`bevy_tilemap::entity`]::[`TileMapComponents`], the components
//! for spawning with a TileMap.
//! * [`bevy_tilemap::map`]::[`TileMap`], the core object that is
//! used for virtually everything in this library.
//! * [`bevy_tilemap::tile`]::{[`Tile`], [`Tiles`]}, a sprite tile which
//! holds minimal amount of data for small data sizes. Used in the `TileMap`.
//! Tiles helps set tiles.
//! * [`bevy_tilemap`]::[`ChunkTilesPlugin`], the main plugin with
//! a collection of systems, components and assets to be used in a Bevy app.
//!
//! [`bevy_tilemap::dimensions`]: crate::dimensions
//! [`bevy_tilemap::chunk`]: crate::chunk
//! [`bevy_tilemap::entity`]: crate::entity
//! [`bevy_tilemap::map`]: crate::map
//! [`bevy_tilemap::tile`]: crate::tile
//! [`bevy_tilemap`]: crate

/// The 0.2 prelude version of Bevy Tilemap.
pub mod v0_2 {
    pub use crate::{
        chunk::LayerKind,
        entity::TileMapComponents,
        tile::{Tile, Tiles},
        tilemap::Tilemap,
        ChunkTilesPlugin,
    };
}

#[cfg(not(v0_1))]
pub use v0_2::*;
