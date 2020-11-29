//! # Bevy Tilemap Sprite
//!
//! Library providing render pipeline, storage of sprite sheets,

#![no_implicit_prelude]
// clippy
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
// rustc
#![deny(dead_code, missing_docs, unused_imports)]

extern crate bevy;
extern crate bevy_tilemap_types;
extern crate rectangle_pack;
extern crate std;

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
    // Having to add this is a bug which is fixed in next Bevy (v > 0.3)
    pub(crate) use ::bevy;
    use ::bevy::{
        app as bevy_app, asset as bevy_asset, math as bevy_math, render as bevy_render,
        sprite as bevy_sprite, type_registry as bevy_type_registry, utils as bevy_utils,
    };

    pub(crate) use self::{
        bevy_app::{AppBuilder, Plugin},
        bevy_asset::{AddAsset, Assets, Handle},
        bevy_math::Vec2,
        bevy_render::{
            renderer::RenderResources,
            texture::{Texture, TextureFormat},
        },
        bevy_sprite::Rect,
        bevy_type_registry::TypeUuid,
        bevy_utils::HashMap,
    };

    pub(crate) use ::bevy_tilemap_types::{
        dimension::{Dimension2, DimensionError},
        point::Point2,
    };

    pub(crate) use ::rectangle_pack::{
        contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, PackedLocation,
        RectToInsert, RectanglePackError, TargetBin,
    };

    pub(crate) use ::std::{
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
    pub(crate) use ::std::{assert, write};

    #[allow(unused_imports)]
    pub(crate) use ::std::panic;

    #[cfg(debug_assertions)]
    #[allow(unused_imports)]
    pub(crate) use ::std::println;
}
