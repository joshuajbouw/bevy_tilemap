//! The tilemap events.

use crate::lib::*;

#[derive(Debug)]
/// Events that can happen to chunks.
pub enum TilemapChunkEvent {
    /// An event when a chunk needs to be spawned.
    Spawned {
        /// The point to get the correct chunk to spawn.
        point: Point2,
    },
    /// An event when a chunk has been modified and needs to reload its layer.
    Modified {
        /// The chunk point that had been modified.
        point: Point2,
    },
    /// An event when a chunk needs to be despawned.
    Despawned {
        /// The point of the chunk to despawn.
        point: Point2,
    },
}
