use crate::{dimension::Dimension2, lib::*};
use bevy::render::pipeline::PrimitiveTopology;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) struct ChunkMesh {
    dimensions: Dimension2,
}

impl ChunkMesh {
    pub(crate) const ATTRIBUTE_TILE_INDEX: &'static str = "Vertex_Tile_Index";
    pub(crate) const ATTRIBUTE_TILE_COLOR: &'static str = "Vertex_Tile_Color";

    pub(crate) fn new(dimensions: Dimension2) -> ChunkMesh {
        ChunkMesh { dimensions }
    }
}

impl From<ChunkMesh> for Mesh {
    fn from(chunk_mesh: ChunkMesh) -> Mesh {
        let chunk_width = chunk_mesh.dimensions.width() as i32;
        let chunk_height = chunk_mesh.dimensions.height() as i32;
        let step_size_x = 1. / chunk_width as f32;
        let step_size_y = 1. / chunk_height as f32;
        let start_x = chunk_width as f32 * -0.5;
        let start_y = chunk_height as f32 * -0.5;
        let mut vertices = Vec::with_capacity((chunk_width * chunk_height) as usize * 4);
        for y in 0..chunk_height {
            for x in 0..chunk_width {
                let y = start_y + y as f32;
                let x = start_x + x as f32;
                vertices.push([x * step_size_x, y * step_size_y, 0.0]);
                vertices.push([x * step_size_x, y * step_size_y + step_size_y, 0.0]);
                vertices.push([
                    x * step_size_x + step_size_x,
                    y * step_size_y + step_size_y,
                    0.0,
                ]);
                vertices.push([x * step_size_x + step_size_x, y * step_size_y, 0.0]);
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
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices.into());
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, tile_indexes.into());
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, tile_colors.into());

        mesh
    }
}
