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

#[allow(deprecated)]
#[deprecated(
    since = "0.2.0",
    note = "please do not use this prelude API, will be removed by v0.3.0"
)]
pub mod v0_1 {
    //! The 0.1 prelude version of Bevy Tilemap.
    //!
    //! Since there is not a lot public declarations and much of the library is
    //! mostly private API, everything that you would likely use is included in
    //! here. The only things that are not included are the [`ToCoord2`] and
    //! [`ToCoord3`] traits in the [`coord`] module. It is quite unlikely that you
    //! would use them in normal usage unless it is for your own personal use.
    //!
    //! [`ToCoord2`]: crate::coord::ToCoord2
    //! [`ToCoord3`]: crate::coord::ToCoord3
    //! [`coord`]: crate::coord
    //!
    //! # Prelude contents
    //!
    //! The current version of this prelude (version 0) is located in
    //! [`bevy_tilemap::prelude::v0`], and re-exports the following.
    //!
    //! * [`bevy_tilemap::chunk`]::[`LayerKind`], the only public part
    //! of `chunk` module is the kind of layer you need to specify to create.
    //! * [`bevy_tilemap::dimensions`]::{[`Dimensions2`], [`Dimensions3`]}.
    //! The dimensions traits provide additional methods to anything that can be
    //! made into the 2nd or 3rd dimension.
    //! * [`bevy_tilemap::entity`]::[`TileMapComponents`], the components
    //! for spawning with a TileMap.
    //! * [`bevy_tilemap::map`]::[`TileMap`], the core object that is
    //! used for virtually everything in this library.
    //! * [`bevy_tilemap::tile`]::[`Tile`], a sprite tile which holds
    //! minimal amount of data for small data sizes. Used in the `TileMap`.
    //! * [`bevy_tilemap::tile_setter`]::[`TileSetter`], a growable heap
    //! used to make setting tiles in a `TileMap` easier.
    //! * [`bevy_tilemap`]::[`ChunkTilesPlugin`], the main plugin with
    //! a collection of systems, components and assets to be used in a Bevy app.
    //!
    //! [`bevy_tilemap::dimensions`]: crate::dimensions
    //! [`bevy_tilemap::chunk`]: crate::chunk
    //! [`bevy_tilemap::entity`]: crate::entity
    //! [`bevy_tilemap::map`]: crate::map
    //! [`bevy_tilemap::tile`]: crate::tile
    //! [`bevy_tilemap::tile_setter`]: crate::tile_setter
    //! [`bevy_tilemap`]: crate

    pub use crate::{
        chunk::LayerKind,
        dimensions::deprecated::{Dimensions2, Dimensions3},
        entity::TileMapComponents,
        tile::Tile,
        tile_setter::TileSetter,
        tilemap::Tilemap as TileMap,
        ChunkTilesPlugin,
    };
}

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

pub use v0_2::*;
