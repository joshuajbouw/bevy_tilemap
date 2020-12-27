use crate::lib::*;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// The mesh of a chunk layer.
pub struct ChunkMesh {
    /// The dimensions of the chunk in pixels.
    dimensions: Dimension2,
}

impl ChunkMesh {
    /// Vertex attribute of the tile's index.
    pub(crate) const ATTRIBUTE_TILE_INDEX: &'static str = "Vertex_Tile_Index";
    /// Vertex attribute of the tile's color.
    pub(crate) const ATTRIBUTE_TILE_COLOR: &'static str = "Vertex_Tile_Color";

    /// Constructs a new chunk mesh.
    pub(crate) fn new(dimensions: Dimension2) -> ChunkMesh {
        ChunkMesh { dimensions }
    }
}

impl From<&ChunkMesh> for Mesh {
    fn from(chunk_mesh: &ChunkMesh) -> Mesh {
        let chunk_width = chunk_mesh.dimensions.width as i32;
        let chunk_height = chunk_mesh.dimensions.height as i32;

        let mut vertices = Vec::with_capacity((chunk_width * chunk_height) as usize * 4);
        for y in 0..chunk_height {
            for x in 0..chunk_width {
                let y0 = y as f32 - chunk_height as f32 / 2.0;
                let y1 = (y + 1) as f32 - chunk_height as f32 / 2.0;
                let x0 = x as f32 - chunk_width as f32 / 2.0;
                let x1 = (x + 1) as f32 - chunk_width as f32 / 2.0;

                vertices.push([x0, y0, 0.0]);
                vertices.push([x0, y1, 0.0]);
                vertices.push([x1, y1, 0.0]);
                vertices.push([x1, y0, 0.0]);
            }
        }

        let indices = Indices::U32(
            (0..(chunk_width * chunk_height) as u32)
                .flat_map(|i| {
                    let i = i * 4;
                    vec![i, i + 2, i + 1, i, i + 3, i + 2]
                })
                .collect(),
        );

        let tile_indexes = vec![0.; vertices.len()];
        let tile_colors: Vec<[f32; 4]> = vec![Color::WHITE.into(); vertices.len()];

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, tile_indexes);
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, tile_colors);

        mesh
    }
}
