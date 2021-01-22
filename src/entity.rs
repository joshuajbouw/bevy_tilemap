//! Bundles of components for spawning entities.

use crate::{
    lib::{Bundle, *},
    Tilemap,
};

/// A component bundle for `Tilemap` entities.
#[derive(Debug, Bundle)]
pub struct TilemapBundle {
    /// A `Tilemap` which maintains chunks and its tiles.
    pub tilemap: Tilemap,
    /// The transform location in a space for a component.
    pub transform: Transform,
    /// The global transform location in a space for a component.
    pub global_transform: GlobalTransform,
}
