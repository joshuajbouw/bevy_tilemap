//! ![](https://github.com/joshuajbouw/bevy_tilemap/raw/master/docs/img/logo.gif)
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
//! * Square and hex tiles.
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
//! bevy_tilemap = { version = "0.2", features = ["serialize"] }
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

#![doc(html_root_url = "https://docs.rs/bevy_tilemap/0.3.0")]
// This was broken even further and no longer will work at all with the previous
// workaround. There is a fix, might be sometime for it to be included though.
// Even then, it is just a warning. For now, including it per module seems to
// fix it.
// See: https://github.com/rust-lang/rust/pull/80372
// #![no_implicit_prelude]

// rustc / rustdoc
#![warn(missing_docs, private_doc_tests)]
#![deny(dead_code, unused_imports)]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(
    clippy::cast_lossless,
    clippy::decimal_literal_representation,
    clippy::else_if_without_else,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::panic_in_result_fn,
    clippy::print_stdout,
    clippy::unwrap_in_result
)]

#[cfg(feature = "types")]
pub extern crate bevy_tilemap_types;

#[cfg(feature = "types")]
#[doc(inline)]
pub use bevy_tilemap_types::dimension;
#[cfg(feature = "types")]
#[doc(inline)]
pub use bevy_tilemap_types::point;

#[no_implicit_prelude]
pub mod default_plugin;
// pub mod auto_tile;
#[no_implicit_prelude]
pub mod chunk;
/// Bundles of components for spawning entities.
#[no_implicit_prelude]
pub mod entity;
/// Meshes for rendering to vertices.
#[no_implicit_prelude]
mod mesh;
#[no_implicit_prelude]
pub mod prelude;
/// Files and helpers for rendering.
#[no_implicit_prelude]
pub mod render;
/// The stages for the tilemap in the bevy app.
#[no_implicit_prelude]
pub mod stage {
    /// The tilemap stage, set to run before `POST_UPDATE` stage.
    pub const TILEMAP: &str = "tilemap";
}
/// Tile traits to implement for a custom tile.
#[no_implicit_prelude]
pub mod tile;
/// Map traits to implement for a custom map and a basic struct for use.
#[no_implicit_prelude]
pub mod tilemap;

use crate::{lib::*, render::TilemapRenderGraphBuilder, tilemap::Tilemap};

/// The Bevy Tilemap 2D main plugin.
#[derive(Default)]
pub struct Tilemap2DPlugin;

impl Plugin for Tilemap2DPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<Tilemap>()
            .add_stage_before(
                app_stage::POST_UPDATE,
                stage::TILEMAP,
                SystemStage::parallel(),
            )
            .add_system_to_stage(
                stage::TILEMAP,
                crate::tilemap::tilemap_auto_configure.system(),
            )
            .add_system_to_stage(stage::TILEMAP, crate::tilemap::tilemap.system())
            .add_system_to_stage(stage::TILEMAP, crate::chunk::chunk_update.system());

        let resources = app.resources_mut();
        let mut render_graph = resources
            .get_mut::<RenderGraph>()
            .expect("`RenderGraph` is missing.");
        render_graph.add_tilemap_graph(resources);
    }
}

/// A custom prelude around everything that we only need to use.
#[no_implicit_prelude]
mod lib {
    extern crate bevy_app;
    extern crate bevy_asset;
    extern crate bevy_core;
    extern crate bevy_ecs;
    extern crate bevy_math;
    extern crate bevy_reflect;
    extern crate bevy_render;
    extern crate bevy_sprite;
    extern crate bevy_tilemap_types;
    extern crate bevy_transform;
    extern crate bevy_utils;
    pub extern crate bitflags;
    #[cfg(feature = "serde")]
    extern crate serde;
    extern crate std;

    pub use self::{
        bevy_app::{
            stage as app_stage, AppBuilder, Events, Plugin, PluginGroup, PluginGroupBuilder,
        },
        bevy_asset::{AddAsset, Assets, Handle, HandleId, HandleUntyped},
        bevy_core::{Byteable, Bytes},
        bevy_ecs::{
            Bundle, Changed, Commands, Entity, IntoSystem, Query, Res, ResMut, Resources,
            SystemStage, TypeInfo,
        },
        bevy_math::{Vec2, Vec3},
        bevy_reflect::{TypeUuid, Uuid},
        bevy_render::{
            color::Color,
            draw::{Draw, Visible},
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
            components::{GlobalTransform, Parent, Transform},
            hierarchy::BuildChildren,
        },
        bevy_utils::{HashMap, HashSet},
    };

    pub use bevy_tilemap_types::{
        dimension::{Dimension2, DimensionError},
        point::Point2,
    };

    pub use crate::bitflags::*;

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
        ops::{Deref, FnMut, FnOnce},
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
