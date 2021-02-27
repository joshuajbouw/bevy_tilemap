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
        /// The layers that had been modified.
        layers: HashMap<usize, Point2>,
    },
    /// An event when a chunk needs to be despawned.
    Despawned {
        /// The point of the chunk to despawn.
        point: Point2,
    },
}

#[cfg(feature = "bevy_rapier2d")]
#[derive(Debug)]
/// Events that can happen to collisions.
pub enum TilemapCollisionEvent {
    /// An event when a collision needs to be spawned.
    Spawned {
        /// The chunk point that needs a collision spawned.
        chunk_point: Point2,
        /// The point in the chunk that needs a collision spawned.
        tiles: Vec<Tile<Point3>>,
    },
    /// An event when a collision needs to be despawned.
    Despawned {
        /// The chunk point that needs a collision spawned.
        chunk_point: Point2,
        /// The point in the chunk that needs a collision spawned.
        tiles: Vec<Tile<Point3>>,
    },
}
