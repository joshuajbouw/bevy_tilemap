use crate::{dimensions::Dimensions3, lib::*};
use bevy::render::pipeline::PrimitiveTopology;

pub struct ChunkMesh {
    width: u32,
    height: u32,
}

impl ChunkMesh {
    pub const ATTRIBUTE_TILE_INDEX: &'static str = "Vertex_Tile_Index";
    pub const ATTRIBUTE_TILE_COLOR: &'static str = "Vertex_Tile_Color";

    pub fn new(dimensions: Vec3) -> ChunkMesh {
        ChunkMesh {
            width: dimensions.width() as u32,
            height: dimensions.height() as u32,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl From<ChunkMesh> for Mesh {
    fn from(chunk_mesh: ChunkMesh) -> Mesh {
        // let extent_x = chunk_mesh.size().x() / 2.0;
        // let extent_y = chunk_mesh.size().y() / 2.0;
        // let tile_width = chunk_mesh.size().x() / chunk_mesh.width() as f32;
        // let tile_height = chunk_mesh.size().y() / chunk_mesh.height() as f32;
        // let capacity = (chunk_mesh.height() + 1) * (chunk_mesh.width() + 1);
        //
        // let mut positions: Vec<[f32; 3]> = Vec::with_capacity(capacity);
        // let mut normals: Vec<[f32; 3]> = Vec::with_capacity(capacity);
        // // This is considered "flipped"
        // for y in 0..chunk_mesh.height() {
        //     for x in 0..chunk_mesh.width() {
        //         let x_coord = x as f32 * tile_width;
        //         let y_coord = y as f32 * tile_height;
        //
        //         let north_west = Vec2::new(-extent_x + x_coord, extent_y + y_coord);
        //         positions.push([north_west.x(), north_west.y(), 0.]);
        //         if y == chunk_mesh.height() && x == chunk_mesh.width {
        //             let south_east = Vec2::new(extent_x + x_coord, -extent_y + y_coord);
        //             positions.push([south_east.x(), south_east.y(), 0.]);
        //         } else if x == chunk_mesh.width() {
        //             let north_east = Vec2::new(extent_x + x_coord, extent_y + y_coord);
        //             positions.push([north_east.x(), north_east.y(), 0.]);
        //         } else if y == chunk_mesh.height() {
        //             let south_west = Vec2::new(-extent_x + x_coord, -extent_y + y_coord);
        //             positions.push([south_west.x(), south_west.y(), 0.]);
        //         }
        //
        //         let norm = vec![[0., 0., 1.]; 4];
        //         normals.extend(norm.iter());
        //     }
        // }
        // let indices = Indices::U32(
        //     (0..(chunk_mesh.width() * chunk_mesh.height()) as u32)
        //         .flat_map(|i| {
        //             let i = i * 4;
        //             vec![i, i + 2, i + 1, i, i + 3, i + 2]
        //         })
        //         .collect(),
        // );

        // let chunk_width = 2 as i32;
        // let chunk_height = 2 as i32;
        let chunk_width = chunk_mesh.width() as i32;
        let chunk_height = chunk_mesh.height() as i32;
        let step_size_x = 1. / chunk_width as f32;
        let step_size_y = 1. / chunk_height as f32;
        let mut vertices = Vec::with_capacity((chunk_width * chunk_height) as usize * 4);
        for y in (-chunk_height / 2)..(chunk_height / 2) {
            for x in (-chunk_width / 2)..(chunk_width / 2) {
                println!("({},{})", x, y);
                vertices.push([
                    x as f32 * step_size_x - step_size_x / 2.0,
                    y as f32 * step_size_y - step_size_y / 2.0,
                    0.0,
                ]);
                vertices.push([
                    x as f32 * step_size_x - step_size_x / 2.0,
                    y as f32 * step_size_y + step_size_y / 2.0,
                    0.0,
                ]);
                vertices.push([
                    x as f32 * step_size_x + step_size_x / 2.0,
                    y as f32 * step_size_y + step_size_y / 2.0,
                    0.0,
                ]);
                vertices.push([
                    x as f32 * step_size_x + step_size_x / 2.0,
                    y as f32 * step_size_y - step_size_y / 2.0,
                    0.0,
                ]);
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
        // for y in (0..chunk_height).rev() {
        //     for x in 0..chunk_width {
        //         positions.push([
        //             (x as f32 - 0.5 * chunk_width as f32) * step_size,
        //             (y as f32 - 0.5 * chunk_height as f32) * step_size,
        //             0.,
        //         ]);
        //         positions.push([
        //             (x as f32 - 0.5 * chunk_width as f32) * step_size,
        //             (y as f32 + 0.5 * chunk_height as f32) * step_size,
        //             0.,
        //         ]);
        //         positions.push([
        //             (x as f32 + 0.5 * chunk_width as f32) * step_size,
        //             (y as f32 + 0.5 * chunk_height as f32) * step_size,
        //             0.,
        //         ]);
        //         positions.push([
        //             (x as f32 + 0.5 * chunk_width as f32) * step_size,
        //             (y as f32 - 0.5 * chunk_height as f32) * step_size,
        //             0.,
        //         ]);
        //         normals.extend([normal; 4].iter());
        //     }
        // }
        //
        // for y in (0..=chunk_height).rev() {
        //     for x in 0..=chunk_width {
        //         if y != 0 && x != chunk_width {
        //             let i: u32 = (chunk_height - y) * (chunk_width + 1) + x;
        //             indices.extend_from_slice(&[
        //                 i + 1,
        //                 i,
        //                 i + chunk_width + 1,
        //                 i + chunk_width + 1,
        //                 i + chunk_width + 2,
        //                 i + 1,
        //             ]);
        //         }
        //     }
        // }
        // println!("indices: {:?}", indices);

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
