use crate::lib::*;

#[derive(Component, Clone, PartialEq, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component, Deserialize, PartialEq)]
/// The mesh of a chunk layer.
pub struct ChunkMesh {
    /// The indices of a chunk's mesh.
    pub(crate) indices: Vec<u32>,
    /// The vertices of a chunk's mesh.
    pub(crate) vertices: Vec<Vec3>,
}

impl ChunkMesh {
    /// Vertex attribute of the tile's index.
    pub(crate) const ATTRIBUTE_TILE_INDEX: &'static str = "Vertex_Tile_Index";
    /// Vertex attribute of the tile's color.
    pub(crate) const ATTRIBUTE_TILE_COLOR: &'static str = "Vertex_Tile_Color";

    /// Constructs a new chunk mesh.
    pub(crate) fn new(dimensions: Dimension3, layers: u32, z_offset: Vec2) -> ChunkMesh {
        let layers = layers as i32;
        let chunk_width = dimensions.width as i32;
        let chunk_height = dimensions.height as i32;
        let chunk_depth = dimensions.depth as i32;
        let mut vertices = Vec::with_capacity((chunk_width * chunk_height) as usize * 4);
        for z in 0..chunk_depth {
            for l in 0..layers {
                for y in 0..chunk_height {
                    for x in 0..chunk_width {
                        let offset_y = z_offset.y * z as f32;
                        let offset_x = z_offset.x * z as f32;
                        let y0 = y as f32 - chunk_height as f32 / 2.0 + offset_y;
                        let y1 = (y + 1) as f32 - chunk_height as f32 / 2.0 + offset_y;
                        let x0 = x as f32 - chunk_width as f32 / 2.0 + offset_x;
                        let x1 = (x + 1) as f32 - chunk_width as f32 / 2.0 + offset_x;

                        let depth = ((z * l) + l) as f32;
                        vertices.push(Vec3::new(x0, y0, depth));
                        vertices.push(Vec3::new(x0, y1, depth));
                        vertices.push(Vec3::new(x1, y1, depth));
                        vertices.push(Vec3::new(x1, y0, depth));
                    }
                }
            }
        }

        let indices = (0..(chunk_width * chunk_height * layers * chunk_depth) as u32)
            .flat_map(|i| {
                let i = i * 4;
                vec![i, i + 2, i + 1, i, i + 3, i + 2]
            })
            .collect::<Vec<_>>();

        info!("mesh vertices: {}", vertices.len());

        ChunkMesh { indices, vertices }
    }
}

impl From<&ChunkMesh> for Mesh {
    fn from(chunk_mesh: &ChunkMesh) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let vertices: Vec<[f32; 3]> = chunk_mesh
            .vertices
            .clone()
            .iter()
            .map(|v| [v.x, v.y, v.z])
            .collect();
        mesh.set_indices(Some(Indices::U32(chunk_mesh.indices.clone())));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        mesh
    }
}
