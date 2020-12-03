//! The Bevy Tilemap sprite prelude.
//!
//! # Prelude contents
//!
//! The current version of this prelude (version 0) is located in
//! [`bevy_tilemap_sprite::prelude::v0`], and re-exports the following.
//!
//! * [`bevy_tilemap_sprite::sprite_sheet`]::{[`SpriteSheet`], [`SpriteSheetBuilder`]},
//! a sprite sheet and a builder both used to construct sprite sheets.
//!
//! [`bevy_tilemap_sprite::prelude::v0`]: crate::prelude::v0
//! [`bevy_tilemap_sprite::sprite_sheet`]: crate::sprite_sheet

/// The v0 prelude version of Bevy Tilemap Sprite.
pub mod v0 {
    pub use crate::sprite_sheet::{SpriteSheet, SpriteSheetBuilder};
}

pub use v0::*;
