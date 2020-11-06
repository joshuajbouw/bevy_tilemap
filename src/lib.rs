//! # Bevy Tilemap
//!
//! Bevy Tilemap allows for Bevy native tile maps to be created with chunk based loading
//! efficiently and generically.
//!
//! Through the use of the power of traits, it is possible to define your own tiles, chunks, and
//! maps exclusive from each other. This allows this to be used as a framework to expand upon it
//! further.
//!
//! ## Design
//!
//! This is not intended to be just another Tile Map. It is meant to be a framework and extensible by
//! design, like Bevy. There is an emphasis placed on generic traits to accomplish that. As well as
//! work done to keep it as close to Bevy API as possible. This allows anyone to create their own tiles,
//! chunks and maps and still retain the speed of a handcrafted multi-threaded chunk loader and tile map.
#![allow(clippy::too_many_arguments)]
// Rustc lints.
#![no_implicit_prelude]
#![forbid(unsafe_code, dead_code)]
#![deny(missing_docs, unused_imports)]

/// Chunk traits to implement for a custom chunk and a basic struct for use.
pub mod chunk;
/// Various coordinate traits used for converting indexes and coordinates.
pub mod coord;
/// Various dimension based traits.
pub mod dimensions;
/// Map traits to implement for a custom map and a basic struct for use.
pub mod map;
/// Tile traits to implement for a custom tile.
pub mod tile;

use crate::lib::*;
pub use crate::{
    chunk::{Chunk, WorldChunk, ChunkSprite},
    map::{TileMap, WorldMap},
    tile::Tile,
};

/// The Bevy Tilemap main plugin.
#[derive(Default)]
pub struct ChunkTilesPlugin<T: Tile, C: Chunk<T>, M: TileMap<T, C>> {
    tile_type: PhantomData<T>,
    chunk_type: PhantomData<C>,
    map_type: PhantomData<M>,
}

impl<T: Tile, C: Chunk<T>, M: TileMap<T, C>> Plugin for ChunkTilesPlugin<T, C, M> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(M::default())
            .add_asset::<C>()
            .add_system_to_stage("post_update", crate::map::map_system::<T, C, M>.system());
    }
}

/// A custom prelude around all the types we need from `std`, `bevy`, and `serde`.
mod lib {
    // Need to add this here as there is a Rust issue surrounding the fact that
    // bevy also uses `no_implicit_prelude`. Without this, it would complain
    // that I am not using `self`, and will refuse to build.
    // See: https://github.com/rust-lang/rust/issues/72381
    pub use ::bevy;
    pub(crate) use ::bevy::{
        app as bevy_app, asset as bevy_asset, ecs as bevy_ecs, math as bevy_math,
        render as bevy_render, sprite as bevy_sprite, tasks as bevy_tasks,
        transform as bevy_transform, type_registry as bevy_type_registry, utils as bevy_utils,
    };

    #[doc(hidden)]
    pub(crate) use self::{
        bevy_app::{AppBuilder, EventReader, Events, Plugin},
        bevy_asset::{AddAsset, Assets, Handle, HandleId},
        bevy_ecs::{Commands, Entity, IntoQuerySystem, Res, ResMut},
        bevy_math::{Vec2, Vec3},
        bevy_render::texture::Texture,
        bevy_sprite::{entity::SpriteComponents, ColorMaterial, Rect, TextureAtlas},
        bevy_tasks::TaskPoolBuilder,
        bevy_transform::components::Transform,
        bevy_type_registry::{TypeUuid, Uuid},
        bevy_utils::{HashMap, HashSet},
    };

    // Need to add this here as there is a Rust issue surrounding the fact that
    // serde also uses `no_implicit_prelude`. Without this, it would complain
    // that I am not using `self`, and will refuse to build.
    // See: https://github.com/rust-lang/rust/issues/72381
    pub use ::serde;
    #[doc(hidden)]
    pub(crate) use ::serde::{Deserialize, Serialize};

    pub(crate) use ::std::{
        boxed::Box,
        clone::Clone,
        convert::{From, Into},
        default::Default,
        fmt::{Debug, Formatter, Result as FmtResult},
        iter::Iterator,
        marker::{PhantomData, Send, Sync},
        ops::Drop,
        option::Option::{self, *},
        result::Result::{self, *},
        slice::{Iter, IterMut},
        string::ToString,
        sync::{Arc, Mutex},
        vec::Vec,
    };

    // Macros
    pub(crate) use ::std::{vec, write};
}
