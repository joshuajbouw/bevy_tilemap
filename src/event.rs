//! The tilemap events.

use crate::{chunk::LayerKind, lib::*, Tile};

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
    /// An event which adds a layer to the chunks.
    AddLayer {
        /// The layer kind to add.
        layer_kind: LayerKind,
        /// Which sprite layer we are adding.
        sprite_layer: usize,
    },
    /// An event which removes a layer from the chunks.
    RemoveLayer {
        /// Which sprite layer we are removing.
        sprite_layer: usize,
    },
    /// An event which spawns tiles into the World.
    SpawnTiles {
        /// The chunk point.
        chunk_point: Point2,
        /// The vector of tiles to spawn into the World.
        tiles: Vec<Tile<Point3>>,
    },
    /// An event which despawns tiles in the World.
    DespawnTiles {
        /// The chunk point.
        chunk_point: Point2,
        /// The vector of tiles to despawn from the World.
        tiles: Vec<Tile<Point3>>,
    },
}
