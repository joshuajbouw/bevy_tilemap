//! # Bevy Tilemap Sprite
//!
//! Library providing render pipeline, storage of sprite sheets,

#![no_implicit_prelude]
// rustc
#![deny(dead_code, unused_imports)]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(clippy::print_stdout, clippy::unwrap_in_result)]
#![deny(
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::decimal_literal_representation,
    clippy::else_if_without_else,
    // clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::panic_in_result_fn,
    clippy::cast_lossless,
    clippy::redundant_pub_crate,
    // clippy::missing_inline_in_public_items,
)]

pub mod prelude;
/// Much like a texture atlas however everything is split into the same size of
/// tiles.
pub mod sprite_sheet;

/// The main plugin for the sprite sheets.
#[derive(Debug, Default)]
pub struct SpriteSheetPlugin;

use crate::{lib::*, sprite_sheet::SpriteSheet};

impl Plugin for SpriteSheetPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<SpriteSheet>();
    }
}

/// A custom prelude around all the types we need from `std`, `bevy`, and `serde`.
mod lib {
    pub extern crate bevy;
    extern crate bevy_tilemap_types;
    extern crate rectangle_pack;
    pub extern crate std;

    // Having to add this is a bug which is fixed in next Bevy (v > 0.3)
    #[allow(unused_imports)]
    use bevy::{
        app as bevy_app, asset as bevy_asset, math as bevy_math, render as bevy_render,
        sprite as bevy_sprite, type_registry as bevy_type_registry, utils as bevy_utils,
    };

    pub use self::{
        bevy_app::{AppBuilder, Plugin},
        bevy_asset::{AddAsset, Assets, Handle},
        bevy_math::Vec2,
        bevy_render::{
            renderer::{
                RenderResource, RenderResourceHints, RenderResourceIterator, RenderResources,
            },
            texture::{Texture, TextureFormat},
        },
        bevy_sprite::Rect,
        bevy_type_registry::{TypeUuid, Uuid},
        bevy_utils::HashMap,
    };

    pub use bevy_tilemap_types::{
        dimension::{Dimension2, DimensionError},
        point::Point2,
    };

    pub use rectangle_pack::{
        contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, PackedLocation,
        RectToInsert, RectanglePackError, TargetBin,
    };

    pub use std::{
        boxed::Box,
        clone::Clone,
        collections::BTreeMap,
        convert::{From, Into},
        default::Default,
        error::Error,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        iter::Iterator,
        option::Option::{self, *},
        result::Result::{self, *},
        vec::Vec,
    };

    // Macros
    pub use std::{assert, write};

    #[allow(unused_imports)]
    pub use std::panic;

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub use std::println;
}
