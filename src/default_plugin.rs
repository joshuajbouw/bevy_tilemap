use crate::lib::*;

/// Implements a plugin group which contains all the plugins.
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(crate::Tilemap2DPlugin::default());
        group.add(crate::sprite_sheet::SpriteSheetPlugin::default());
    }
}
