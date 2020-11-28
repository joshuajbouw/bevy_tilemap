//! ![](https://github.com/joshuajbouw/bevy_tilemap/raw/master/assets/img/logo.gif)
//!
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
#![no_implicit_prelude]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
// rustc
#![deny(dead_code, missing_docs, unused_imports)]

extern crate bevy;
extern crate bevy_tilemap_spritesheet;
extern crate bevy_tilemap_types;
extern crate bitflags;
#[cfg(feature = "serde")]
extern crate serde;
extern crate std;

/// The default plugin to be used in Bevy applications.
pub mod default_plugin;
/// Similar to a texture atlas but splits everything into the same size tiles.
pub mod sprite_sheet {
    pub use crate::bevy_tilemap_spritesheet::*;
}
#[cfg(feature = "types")]
/// Common, but optional, types used across Bevy Tilemaps
pub mod types {
    pub use crate::bevy_tilemap_types::*;
}
// pub mod auto_tile;
/// Chunk traits to implement for a custom chunk and a basic struct for use.
pub mod chunk;
/// Bundles of components for spawning entities.
pub mod entity;
/// Meshes for rendering to vertices.
mod mesh;
pub mod prelude;
/// Files and helpers for rendering.
mod render;
/// Tile traits to implement for a custom tile.
pub mod tile;
/// Map traits to implement for a custom map and a basic struct for use.
pub mod tilemap;

use crate::{chunk::Chunk, lib::*, render::TilemapRenderGraphBuilder, tilemap::Tilemap};

/// The Bevy Tilemap 2D main plugin.
#[derive(Default)]
pub struct Tilemap2DPlugin;

impl Plugin for Tilemap2DPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<Tilemap>()
            .add_asset::<Chunk>()
            .add_system_to_stage(
                "post_update",
                crate::tilemap::tilemap_auto_configure.system(),
            )
            .add_system_to_stage("post_update", crate::tilemap::tilemap_system.system())
            .add_system_to_stage("post_update", crate::chunk::chunk_update_system.system());

        let resources = app.resources_mut();
        let mut render_graph = resources
            .get_mut::<RenderGraph>()
            .expect("`RenderGraph` is missing.");
        render_graph.add_tilemap_graph(resources);
    }
}

/// A custom prelude around all the types we need from `std`, `bevy`, and `serde`.
mod lib {
    // Having to add this is a bug which is fixed in next Bevy (v > 0.3)
    pub(crate) use ::bevy;
    use ::bevy::{
        app as bevy_app, asset as bevy_asset, core as bevy_core, ecs as bevy_ecs,
        math as bevy_math, render as bevy_render, sprite as bevy_sprite,
        transform as bevy_transform, type_registry as bevy_type_registry, utils as bevy_utils,
    };

    pub(crate) use self::{
        bevy_app::{AppBuilder, Events, Plugin, PluginGroup, PluginGroupBuilder},
        bevy_asset::{AddAsset, Assets, Handle, HandleId},
        bevy_core::Byteable,
        bevy_ecs::{Bundle, Commands, Entity, IntoQuerySystem, Query, Res, ResMut, Resources},
        bevy_math::{Vec2, Vec3},
        bevy_render::{
            color::Color,
            draw::Draw,
            mesh::{Indices, Mesh},
            pipeline::{
                BlendDescriptor, BlendFactor, BlendOperation, ColorStateDescriptor, ColorWrite,
                CompareFunction, CullMode, DepthStencilStateDescriptor, DynamicBinding, FrontFace,
                PipelineDescriptor, PipelineSpecialization, PrimitiveTopology,
                RasterizationStateDescriptor, RenderPipeline, RenderPipelines,
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
        bevy_utils::{HashMap, HashSet},
    };

    pub(crate) use ::bevy_tilemap_types::{
        dimension::{Dimension2, DimensionError},
        point::Point2,
    };

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
        ops::{FnMut, FnOnce},
        option::Option::{self, *},
        result::Result::{self, *},
        vec::Vec,
    };

    // Macros
    pub(crate) use ::std::{assert_eq, panic, vec, write};

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub(crate) use ::std::println;
}
