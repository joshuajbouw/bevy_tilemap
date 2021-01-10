//! The default plugin to be used in Bevy applications.
//!
//! This contains all the plugins in a single bundle which can be used to start
//! immediately using tilemaps.
//!
//! # Using the default plugin
//! ```no_run
//! use bevy_app::prelude::*;
//! use bevy_tilemap::prelude::*;
//!
//! App::build()
//!     .add_plugins(TilemapDefaultPlugins)
//!     .run()
//! ```

use crate::lib::*;

/// Implements a plugin group which contains all the plugins.
pub struct TilemapDefaultPlugins;

impl PluginGroup for TilemapDefaultPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(crate::Tilemap2DPlugin::default());
    }
}
