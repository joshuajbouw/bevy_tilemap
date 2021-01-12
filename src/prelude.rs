//! The Bevy Tilemap prelude.
//!
//! # Prelude contents
//!
//! The current version of this prelude (version 0) is located in
//! [`bevy_tilemap::prelude::v0`], and re-exports the following.
//!
//! * [`bevy_tilemap::chunk`]::[`LayerKind`], the only public part
//! of `chunk` module is the kind of layer you need to specify to create.
//! * [`bevy_tilemap::default_plugin`]::[`TilemapDefaultPlugins`], the
//! default plugins for the library.
//! * [`bevy_tilemap::entity`]::[`TilemapBundle`], the component bundle
//! for spawning with a Tilemap.
//! * [`bevy_tilemap::tile`]::[`Tile`], a sprite tile which
//! holds minimal amount of data for small data sizes.
//! * [`bevy_tilemap::tilemap`]::{[`Tilemap`], [`TilemapBuilder`]},
//! the core object that is used for virtually everything in this library.
//! * [`bevy_tilemap`]::[`Tilemap2DPlugin`], the main plugin with
//! a collection of systems, components and assets to be used in a Bevy app.
//!
//! [`bevy_tilemap::prelude::v0`]: crate::prelude::v0
//! [`bevy_tilemap::default_plugin`]: crate::default_plugin
//! [`bevy_tilemap::chunk`]: crate::chunk
//! [`bevy_tilemap::entity`]: crate::entity
//! [`bevy_tilemap::tile`]: crate::tile
//! [`bevy_tilemap::tilemap`]: crate::tilemap
//! [`bevy_tilemap`]: crate

/// Version 0 prelude.
pub mod v0 {
    pub use crate::{
        chunk::LayerKind,
        default_plugin::TilemapDefaultPlugins,
        entity::TilemapBundle,
        render::GridTopology,
        tile::Tile,
        tilemap::{Tilemap, TilemapBuilder},
        Tilemap2DPlugin,
    };
}

pub use v0::*;
