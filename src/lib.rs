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
#![no_implicit_prelude]
#![deny(dead_code, missing_docs, unused_imports)]

/// Chunk traits to implement for a custom chunk and a basic struct for use.
pub mod chunk;
/// Various dimension based traits.
mod dimension;
/// Bundles of components for spawning entities.
pub mod entity;
/// Meshes for use in rendering.
pub(crate) mod mesh;
pub(crate) mod point;
pub mod prelude;
/// Files and helpers for rendering.
pub(crate) mod render;
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
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
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
        fmt::{Debug, Formatter, Result as FmtResult},
        iter::{Extend, Iterator},
        ops::{FnMut, FnOnce},
        option::Option::{self, *},
        result::Result::{self, *},
        vec::Vec,
    };

    // Macros
    pub(crate) use ::std::{panic, vec, write};
}
