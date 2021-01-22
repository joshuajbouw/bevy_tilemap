//! The tilemap events.

use crate::lib::*;

#[derive(Debug)]
/// Events that can happen to chunks.
pub enum TilemapEvent {
    /// An event when a chunk needs to be spawned.
    Spawned {
        /// The point to get the correct chunk to spawn.
        point: Point2,
    },
    /// An event when a chunk has been modified and needs to reload its layer.
    Modified {
        /// The layers that had been modified.
        layers: HashMap<usize, Entity>,
    },
    /// An even when a chunk needs to be despawned.
    Despawned {
        /// The entities that need to be despawned.
        entities: Vec<Entity>,
        /// The point of the chunk to despawn.
        point: Point2,
    },
}

// pub enum TilemapCollisionEvent {
//     /// An event which indicates that a chunk was spawned.
//     SpawnedChunk {
//         /// The chunk point that needs collisions created.
//         chunk_point: Point2,
//     },
//     Spawned {
//         /// The chunk point that needs a collision spawned.
//         chunk_point: Point2,
//         /// The point in the chunk that needs a collision spawned.
//         point: Point2,
//     },
//     Despawned {
//         /// The entities that need to be despawned.
//         entities: Vec<Entity>,
//         /// The point of the chunk to despawn collisions.
//         point: Point2,
//     },
// }
