//! [![](https://github.com/joshuajbouw/bevy_tilemap/blob/master/assets/img/logo.gif)]
//! # Bevy Tilemap
//!
//! Bevy Tilemap allows for Bevy native batch-rendered tiles in maps to be
//! constructed with chunk based loading, efficiently.
//!
//! Simple yet refined in its implementation, it is meant to attach to other
//! extensible plugins that can enhance its functionality further. Hand-crafted
//! tilemaps with an attentive focus on performance, and low data usage.
//!
//! ## Features
//! * Perfect for game jams.
//! * Easy to use and stable API with thorough documentation.
//! * Endless or constrained tilemaps.
//! * Batched rendering of many tiles.
//!
//! ## Design
//! This is not intended to be just another Tile Map. It is meant to be a
//! framework and extensible by design, like Bevy. As well as work done to keep
//! it as close to Bevy API as possible while keeping in mind of Rust API best
//! practices. It is not meant to be complicated and created to be simple to use
//! but give enough functionality to advanced users.
//!
//! Less time fiddling, more time building.
//!
//! # Constructing a basic tilemap, setting tiles, and spawning.
//!
//! Bevy Tilemap makes it easy to quickly implement a tilemap if you are in a
//! rush or want to build a conceptual game.
//!
//! ```
//! use bevy_tilemap::prelude::*;
//! use bevy::asset::HandleId;
//! use bevy::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::new(texture_atlas_handle);
//!
//! // Coordinate point with Z order.
//! let point = (16, 16, 0);
//! let tile_index = 0;
//! tilemap.set_tile(point, tile_index);
//!
//! tilemap.spawn_chunk_containing_point(point);
//! ```
//!
//! # Constructing a more advanced tilemap.
//!
//! For most cases, it is preferable to construct a tilemap with explicit
//! parameters. For that you would use a [`Builder`].
//!
//! ```
//! use bevy_tilemap::prelude::*;
//! use bevy::asset::HandleId;
//! use bevy::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::builder()
//!     .texture_atlas(texture_atlas_handle)
//!     .chunk_dimensions(64, 64)
//!     .tile_dimensions(8, 8)
//!     .dimensions(32, 32)
//!     .add_layer(LayerKind::Dense, 0)
//!     .add_layer(LayerKind::Sparse, 1)
//!     .add_layer(LayerKind::Sparse, 2)
//!     .z_layers(3)
//!     .build()
//!     .unwrap();
//! ```
//!
//! The above example outlines all the current possible builder methods. What is
//! neat is that if more layers are accidentally set than z_layer set, it will
//! use the layer length instead. Much more features are planned including
//! automated systems that will enhance the tilemap further.
//!
//! # Setting tiles
//!
//! There are two methods to set tiles in the tilemap. The first is single tiles
//! at a time which is acceptable for tiny updates such as moving around
//! characters. The second being bulk setting many tiles at once.
//!
//! If you expect to move multiple tiles a frame, **always** use the [`Tiles`]
//! map and set it with [`set_tiles`]. A single event is created with all
//! tiles if set this way.
//!
//! ```
//! use bevy_tilemap::prelude::*;
//! use bevy::asset::HandleId;
//! use bevy::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::new(texture_atlas_handle);
//!
//! // Prefer this
//! let mut tiles = Tiles::default();
//! for y in 0..31 {
//!     for x in 0..31 {
//!         tiles.insert((x, y, 0), 0.into());
//!     }
//! }
//!
//! tilemap.set_tiles(&mut tiles);
//!
//! // Over this...
//! for y in 0..31 {
//!     for x in 0..31 {
//!         tilemap.set_tile((x, y, 0), 0);
//!     }
//! }
//! ```
//!
//! # Serde support
//!
//! Optionally serde is supported through the use of features.
//!
//! ```toml
//! [dependencies]
//! bevy_tilemap = { version = "0.2", features = ["serde"] }
//! ```
#![doc(html_root_url = "https://bevy_tilemap/0.2.0-pre.1")]
#![no_implicit_prelude]
// clippy
#![allow(clippy::too_many_arguments)]
// rustc
#![deny(dead_code, missing_docs, unused_imports)]

/// Chunk traits to implement for a custom chunk and a basic struct for use.
pub mod chunk;
/// Various dimension based traits.
mod dimension;
/// Bundles of components for spawning entities.
pub mod entity;
/// Meshes for use in rendering.
mod mesh;
/// Points used for helping with coordinates.
pub mod point;
pub mod prelude;
/// Files and helpers for rendering.
mod render;
/// Tile traits to implement for a custom tile.
pub mod tile;
/// Map traits to implement for a custom map and a basic struct for use.
pub mod tilemap;

use crate::{chunk::Chunk, lib::*, render::TilemapRenderGraphBuilder, tilemap::Tilemap};

/// The Bevy Tilemap main plugin.
#[derive(Default)]
pub struct ChunkTilesPlugin;

impl Plugin for ChunkTilesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<Tilemap>()
            .add_asset::<Chunk>()
            .add_system_to_stage("post_update", crate::tilemap::map_system.system());

        let resources = app.resources_mut();
        let mut render_graph = resources
            .get_mut::<RenderGraph>()
            .expect("`RenderGraph` is missing.");
        render_graph.add_tilemap_graph(resources);
    }
}

/// A custom prelude around all the types we need from `std`, `bevy`, and `serde`.
mod lib {
    // Need to add this here as there is a Rust issue surrounding the fact that
    // bevy also uses `no_implicit_prelude`. Without this, it would complain
    // that I am not using `self`, and will refuse to build.
    // See: https://github.com/rust-lang/rust/issues/72381
    pub use ::bevy;
    use ::bevy::{
        app as bevy_app, asset as bevy_asset, core as bevy_core, ecs as bevy_ecs,
        math as bevy_math, render as bevy_render, sprite as bevy_sprite,
        transform as bevy_transform, type_registry as bevy_type_registry, utils as bevy_utils,
    };

    pub(crate) use self::{
        bevy_app::{AppBuilder, Events, Plugin},
        bevy_asset::{AddAsset, Assets, Handle, HandleId},
        bevy_core::Byteable,
        bevy_ecs::{Bundle, Commands, Entity, IntoQuerySystem, Query, ResMut, Resources},
        bevy_math::{Vec2, Vec3},
        bevy_render::{
            color::Color,
            draw::Draw,
            mesh::{Indices, Mesh},
            pipeline::{
                BlendDescriptor, BlendFactor, BlendOperation, ColorStateDescriptor, ColorWrite,
                CompareFunction, CullMode, DepthStencilStateDescriptor, DynamicBinding, FrontFace,
                PipelineDescriptor, RasterizationStateDescriptor, RenderPipeline, RenderPipelines,
                StencilStateDescriptor, StencilStateFaceDescriptor,
            },
            render_graph::{base::MainPass, RenderGraph, RenderResourcesNode},
            renderer::{RenderResource, RenderResources},
            shader::{Shader, ShaderStage, ShaderStages},
            texture::TextureFormat,
        },
        bevy_sprite::TextureAtlas,
        bevy_transform::{
            components::{GlobalTransform, Transform},
            hierarchy::BuildChildren,
        },
        bevy_type_registry::{TypeUuid, Uuid},
        bevy_utils::HashMap,
    };

    // Need to add this here as there is a Rust issue surrounding the fact that
    // serde also uses `no_implicit_prelude`. Without this, it would complain
    // that I am not using `self`, and will refuse to build.
    // See: https://github.com/rust-lang/rust/issues/72381
    #[cfg(feature = "serde")]
    pub use ::serde;
    #[cfg(feature = "serde")]
    pub(crate) use ::serde::{Deserialize, Serialize};

    pub(crate) use ::std::{
        self,
        boxed::Box,
        clone::Clone,
        cmp::Ord,
        convert::{AsMut, AsRef, From, Into},
        default::Default,
        error::Error,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        iter::{Extend, IntoIterator, Iterator},
        ops::{Add, AddAssign, Div, DivAssign, FnMut, FnOnce, Mul, MulAssign, Neg, Sub, SubAssign},
        option::Option::{self, *},
        result::Result::{self, *},
        vec::Vec,
    };

    // Macros
    pub(crate) use ::std::{panic, vec, write};
}
