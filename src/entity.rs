//! Bundles of components for spawning entities.

use crate::{lib::*, Tilemap};
use ::bevy_ecs;
use ::std;

/// A component bundle for `Tilemap` entities.
#[derive(Debug, Bundle)]
pub struct TilemapBundle {
    /// A `Tilemap` which maintains chunks and its tiles.
    pub tilemap: Tilemap,
    /// A component that indicates if the component is visible.
    pub visible: Visible,
    /// The transform location in a space for a component.
    pub transform: Transform,
    /// The global transform location in a space for a component.
    pub global_transform: GlobalTransform,
}
