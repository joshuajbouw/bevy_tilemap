use crate::{lib::*, tile::Tile};

/// A component that stores the dimensions of the Chunk for the renderer.
#[derive(Debug, Default, RenderResources, RenderResource)]
#[render_resources(from_self)]
pub struct ChunkDimensions {
    /// The chunk dimensions.
    pub dimensions: Vec3,
}

unsafe impl Byteable for ChunkDimensions {}

impl From<Vec3> for ChunkDimensions {
    fn from(vec: Vec3) -> Self {
        ChunkDimensions { dimensions: vec }
    }
}

/// A chunk which is used internally of the `TileMap`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Chunk {
    /// A mesh handle.
    #[cfg_attr(feature = "serde", serde(skip))]
    mesh: Handle<Mesh>,
    /// A vector of all the tiles in the `TileMap`.
    tiles: Vec<Tile>,
}

impl TypeUuid for Chunk {
    const TYPE_UUID: Uuid = Uuid::from_u128(45182109655678555067446040298151572788);
}

impl Chunk {
    /// Returns a new `WorldChunk`.
    pub(crate) fn new(tiles: Vec<Tile>, mesh: Handle<Mesh>) -> Chunk {
        Chunk { mesh, tiles }
    }

    pub(crate) fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    pub(crate) fn set_tile(&mut self, index: usize, tile: Tile) {
        self.tiles[index] = tile;
    }

    pub(crate) fn tiles_to_renderer_parts(&self) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::tiles_to_renderer_parts(&self.tiles)
    }
}
