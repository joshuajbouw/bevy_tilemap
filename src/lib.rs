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
//! This is not intended to be just another Tilemap. It is meant to be a
//! framework and extensible by design, like Bevy. As well as work done to keep
//! it as close to Bevy API as possible while keeping in mind of Rust API best
//! practices. It is not meant to be complicated and created to be simple to use
//! but give enough functionality to advanced users.
//!
//! Less time fiddling, more time building.
//!
//! # Serde support
//!
//! Optionally serde is supported through the use of features.
//!
//! ```toml
//! [dependencies]
//! bevy_tilemap = { version = "0.2", features = ["serde"] }
//! ```
//!
//! # Extra types feature
//!
//! Internally, the library uses [`Point2`], [`Point3`], [`Dimension2`] and
//! [`Dimension3`] types. This is not part of the official Bevy library and
//! multiple or alternative implementations of them may not be ideal, especially
//! not in the prelude.
//!
//! It is quite important however to `impl Into<T>` for each of them for most
//! public methods. It already has most basic implementations that make sense.
//!
//! However if you would like to use this, please do so.
//!
//! ```toml
//! [dependencies]
//! bevy_tilemap = { version = "0.2", features = ["types"] }
//! ```
//!
//! [`Point2`]: crate::point::Point2
//! [`Point3`]: crate::point::Point3
//! [`Dimension2`]: crate::dimension::Dimension2
//! [`Dimension3`]: crate::dimension::Dimension3

#![no_implicit_prelude]
// rustc
#![warn(missing_doc_code_examples, missing_docs, private_doc_tests)]
#![deny(dead_code, unused_imports)]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(
    clippy::cast_lossless,
    clippy::decimal_literal_representation,
    clippy::else_if_without_else,
    // clippy::indexing_slicing, // TODO: Change back in when Bevy is updated.
    clippy::let_underscore_must_use,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::panic_in_result_fn,
    clippy::print_stdout,
    // clippy::redundant_pub_crate,
    clippy::unwrap_in_result
)]

pub extern crate bevy_tilemap_spritesheet;
#[cfg(feature = "types")]
pub extern crate bevy_tilemap_types;

#[doc(inline)]
pub use bevy_tilemap_spritesheet as sprite_sheet;
#[cfg(feature = "types")]
#[doc(inline)]
pub use bevy_tilemap_types::dimension;
#[cfg(feature = "types")]
#[doc(inline)]
pub use bevy_tilemap_types::point;

pub mod default_plugin;
// pub mod auto_tile;
pub mod chunk;
/// Bundles of components for spawning entities.
pub mod entity;
/// Meshes for rendering to vertices.
mod mesh;
pub mod prelude;
/// Files and helpers for rendering.
pub mod render;
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
            .add_system_to_stage("post_update", crate::tilemap::tilemap_auto_configure)
            .add_system_to_stage("post_update", crate::tilemap::tilemap_system)
            .add_system_to_stage("post_update", crate::chunk::chunk_update_system);

        let resources = app.resources_mut();
        let mut render_graph = resources
            .get_mut::<RenderGraph>()
            .expect("`RenderGraph` is missing.");
        render_graph.add_tilemap_graph(resources);
    }
}

/// A custom prelude around everything that we only need to use.
mod lib {
    pub extern crate bevy;
    pub extern crate bevy_tilemap_types;
    pub extern crate bitflags;
    #[cfg(feature = "serde")]
    pub extern crate serde;
    pub extern crate std;

    // Having to add this is a bug which is fixed in next Bevy (v > 0.3)
    use bevy::{
        app as bevy_app, asset as bevy_asset, core as bevy_core, ecs as bevy_ecs,
        math as bevy_math, reflect as bevy_reflect, render as bevy_render, sprite as bevy_sprite,
        transform as bevy_transform, utils as bevy_utils,
    };

    pub use self::{
        bevy_app::{AppBuilder, Events, Plugin, PluginGroup, PluginGroupBuilder},
        bevy_asset::{AddAsset, Assets, Handle, HandleId},
        bevy_core::{Byteable, Bytes},
        bevy_ecs::{Bundle, Commands, Entity, Query, Res, ResMut, Resources},
        bevy_math::{Vec2, Vec3},
        bevy_reflect::{TypeUuid, Uuid},
        bevy_render::{
            color::Color,
            draw::Draw,
            mesh::{Indices, Mesh},
            pipeline::{
                BlendDescriptor, BlendFactor, BlendOperation, ColorStateDescriptor, ColorWrite,
                CompareFunction, CullMode, DepthStencilStateDescriptor, FrontFace,
                PipelineDescriptor, PrimitiveTopology, RasterizationStateDescriptor,
                RenderPipeline, RenderPipelines, StencilStateDescriptor,
                StencilStateFaceDescriptor,
            },
            render_graph::{base::MainPass, RenderGraph, RenderResourcesNode},
            renderer::{
                RenderResource, RenderResourceIterator, RenderResourceType, RenderResources,
            },
            shader::{Shader, ShaderStage, ShaderStages},
            texture::{Texture, TextureFormat},
        },
        bevy_sprite::TextureAtlas,
        bevy_transform::{
            components::{GlobalTransform, Transform},
            hierarchy::BuildChildren,
        },
        bevy_utils::{HashMap, HashSet},
    };

    pub use crate::bevy_tilemap_types::{
        dimension::{Dimension2, DimensionError},
        point::Point2,
    };

    #[cfg(feature = "serde")]
    pub use serde::{Deserialize, Serialize};

    pub use std::{
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
    pub use std::{assert_eq, panic, vec, write};

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub use std::println;
}
