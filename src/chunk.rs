use crate::{lib::*, mesh::ChunkMesh, tile::Tile};

/// A basic use of the `Chunk` trait that has the bare minimum methods.
///
/// Serde skips the textures and texture_handle field for three reasons:
/// * Handle doesn't support it.
/// * Rect doesn't support it.
/// * Even if the above supported it, there shouldn't be a need to store that
/// information anyways as they are temporary.
#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    #[serde(skip)]
    mesh: Handle<Mesh>,
    /// A vector of all the tiles in the `TileMap`.
    tiles: Vec<Tile>,
}

impl TypeUuid for Chunk {
    const TYPE_UUID: Uuid = Uuid::from_u128(45182109655678555067446040298151572788);
}

impl Chunk {
    /// Returns a new `WorldChunk`.
    ///
    /// # Arguments
    /// * texture_handle - Takes in a `Handle<Texture>` to store for use with
    /// getting the correct texture from Bevy assets.
    pub fn new(
        tile_size: Vec2,
        width: u32,
        height: u32,
        depth: u32,
        meshes: &mut Assets<Mesh>,
    ) -> Chunk {
        let mut sprites = Vec::new();
        for y in 0..width {
            for x in 0..height {
                sprites.push(Rect {
                    min: Vec2::new(x as f32 * tile_size.x(), y as f32 * tile_size.y()),
                    max: Vec2::new(
                        (x + 1) as f32 * tile_size.x(),
                        (y + 1) as f32 * tile_size.y(),
                    ),
                })
            }
        }
        let mesh = meshes.add(Mesh::from(ChunkMesh::new(width, height)));
        Chunk {
            mesh,
            tiles: Vec::with_capacity((width * height * depth) as usize),
        }
    }

    // /// Sets the texture handle with a new one.
    // ///
    // /// # Warning
    // /// This should **only** be used when creating a new `Chunk` from the
    // /// `default` method.
    // pub fn set_texture_handle(&mut self, handle: Handle<Texture>) {
    //     self.texture = handle;
    // }

    // /// Returns a reference to the `Texture` `Handle`.
    // pub fn texture_handle(&self) -> &Handle<Texture> {
    //     &self.texture
    // }

    pub fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    // /// Returns a reference to the `Tile` in the `Chunk`, if it exists.
    // pub fn tile_stack(&self, coord: &Vec3) -> DimensionResult<Option<&Vec<Tile>>> {
    //     let idx = self.encode_coord(coord)?;
    //     Ok(self.tiles[idx].as_ref())
    // }

    pub fn set_tiles(&mut self, tiles: Vec<Tile>) {
        self.tiles = tiles;
    }

    pub fn set_tile(&mut self, index: usize, tile: Tile) {
        self.tiles[index] = tile;
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    pub fn tiles_to_renderer_parts(&self) -> (Vec<f32>, Vec<[f32; 4]>) {
        let mut tile_indexes: Vec<f32> = Vec::with_capacity(self.tiles.len());
        let mut tile_colors: Vec<[f32; 4]> = Vec::with_capacity(self.tiles.len());
        for tile in self.tiles.iter() {
            tile_indexes.push(tile.index() as f32);
            tile_colors.push(tile.color().into());
        }
        (tile_indexes, tile_colors)
    }
}
