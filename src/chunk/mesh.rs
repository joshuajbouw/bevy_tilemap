use crate::lib::*;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
/// The mesh of a chunk layer.
pub struct ChunkMesh {
    /// The dimensions of the chunk in tiles.
    dimensions: Dimension3,
    /// Layers pers Z depth.
    layers: u32,
    /// The offset of the chunk per Z depth.
    z_offset: Vec2,
}

impl ChunkMesh {
    /// Vertex attribute of the tile's index.
    pub(crate) const ATTRIBUTE_TILE_INDEX: &'static str = "Vertex_Tile_Index";
    /// Vertex attribute of the tile's color.
    pub(crate) const ATTRIBUTE_TILE_COLOR: &'static str = "Vertex_Tile_Color";

    /// Constructs a new chunk mesh.
    pub(crate) fn new(dimensions: Dimension3, layers: u32, z_offset: Vec2) -> ChunkMesh {
        ChunkMesh {
            dimensions,
            layers,
            z_offset,
        }
    }
}

impl From<&ChunkMesh> for Mesh {
    fn from(chunk_mesh: &ChunkMesh) -> Mesh {
        let chunk_width = chunk_mesh.dimensions.width as i32;
        let chunk_height = chunk_mesh.dimensions.height as i32;
        let chunk_depth = chunk_mesh.dimensions.depth as i32;
        let chunk_layers = chunk_mesh.layers as i32;
        let z_offset = chunk_mesh.z_offset;

        let mut vertices = Vec::with_capacity((chunk_width * chunk_height) as usize * 4);
        for z in 0..chunk_depth {
            for l in 0..chunk_layers {
                for y in 0..chunk_height {
                    for x in 0..chunk_width {
                        let offset_y = z_offset.y * z as f32;
                        let offset_x = z_offset.x * z as f32;
                        let y0 = y as f32 - chunk_height as f32 / 2.0 + offset_y;
                        let y1 = (y + 1) as f32 - chunk_height as f32 / 2.0 + offset_y;
                        let x0 = x as f32 - chunk_width as f32 / 2.0 + offset_x;
                        let x1 = (x + 1) as f32 - chunk_width as f32 / 2.0 + offset_x;

                        let depth = ((z * l) + l) as f32;
                        vertices.push([x0, y0, depth]);
                        vertices.push([x0, y1, depth]);
                        vertices.push([x1, y1, depth]);
                        vertices.push([x1, y0, depth]);
                    }
                }
            }
        }

        let indices = Indices::U32(
            (0..(chunk_width * chunk_height * chunk_layers * chunk_depth) as u32)
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
