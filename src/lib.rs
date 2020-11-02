#![allow(clippy::too_many_arguments)]
// Rustc lints.
#![no_implicit_prelude]
#![forbid(unsafe_code, dead_code)]
// #![deny(missing_docs, unused_imports)]

pub mod chunk;
pub mod coord;
pub mod dimensions;
pub mod map;
pub mod tile;

use crate::{
    chunk::TileChunk,
    lib::*,
    map::{map_system, TileMap},
    tile::Tile,
};

#[derive(Default)]
pub struct ChunkTilesPlugin<T: Tile, C: TileChunk<T>, M: TileMap<T, C>> {
    tile_type: PhantomData<T>,
    chunk_type: PhantomData<C>,
    map_type: PhantomData<M>,
}

impl<T: Tile, C: TileChunk<T>, M: TileMap<T, C>> Plugin for ChunkTilesPlugin<T, C, M> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(M::default())
            .add_asset::<C>()
            .add_system_to_stage("post_update", map_system::<T, C, M>.system());
    }
}

mod lib {
    pub use ::bevy::{
        self, app as bevy_app, asset as bevy_asset, ecs as bevy_ecs, math as bevy_math,
        render as bevy_render, sprite as bevy_sprite, transform as bevy_transform,
        type_registry as bevy_type_registry, utils as bevy_utils,
    };

    pub use self::{
        bevy_app::{AppBuilder, EventReader, Events, Plugin},
        bevy_asset::{AddAsset, Assets, Handle, HandleId},
        bevy_ecs::{Commands, Entity, IntoQuerySystem, Res, ResMut},
        bevy_math::{Vec2, Vec3},
        bevy_render::texture::Texture,
        bevy_sprite::{entity::SpriteComponents, ColorMaterial, Rect, TextureAtlas},
        bevy_transform::components::Transform,
        bevy_type_registry::{TypeUuid, Uuid},
        bevy_utils::{HashMap, HashMapExt, HashSet},
    };

    pub use ::serde::{Deserialize, Serialize};

    pub use ::std::{
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
        vec::Vec,
    };

    // Macros
    pub use ::std::{println, vec, write};
}
