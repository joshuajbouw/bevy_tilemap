use crate::{
    chunk::{
        entity::{ModifiedLayer, ZOrder},
        mesh::ChunkMesh,
    },
    lib::*,
    Tilemap,
};

/// The chunk update system that is used to set attributes of the tiles and
/// tints if they need updating.
pub(crate) fn chunk_update(
    mut meshes: ResMut<Assets<Mesh>>,
    map_query: Query<&Tilemap>,
    mut chunk_query: Query<(&Parent, &Point2, &ZOrder, &Handle<Mesh>), Changed<ModifiedLayer>>,
) {
    for (parent, point, z_order, mesh_handle) in chunk_query.iter_mut() {
        let tilemap = if let Ok(tilemap) = map_query.get(**parent) {
            tilemap
        } else {
            error!("`Tilemap` is missing, can not update chunk");
            return;
        };
        let chunk = if let Some(chunk) = tilemap.get_chunk(point) {
            chunk
        } else {
            error!("`Chunk` is missing, can not update chunk");
            return;
        };
        let mesh = if let Some(mesh) = meshes.get_mut(mesh_handle) {
            mesh
        } else {
            error!("`Mesh` is missing, can not update chunk");
            return;
        };
        let (indexes, colors) = if let Some((index, colors)) =
            chunk.tiles_to_renderer_parts(z_order.0, tilemap.chunk_dimensions())
        {
            (index, colors)
        } else {
            error!("Tiles are missing, can not update chunk");
            return;
        };
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes);
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors);
    }
}

/// Actual method used to spawn chunks.
fn auto_spawn(
    camera_transform: &Transform,
    tilemap_transform: &Transform,
    tilemap: &mut Tilemap,
    spawn_dimensions: Dimension2,
) {
    let translation = camera_transform.translation - tilemap_transform.translation;
    let point_x = translation.x / tilemap.tile_width() as f32;
    let point_y = translation.y / tilemap.tile_height() as f32;
    let (chunk_x, chunk_y) = tilemap.point_to_chunk_point((point_x as i32, point_y as i32));
    let mut new_spawned: Vec<Point2> = Vec::new();
    let spawn_width = spawn_dimensions.width as i32;
    let spawn_height = spawn_dimensions.height as i32;
    for y in -spawn_width as i32..spawn_width + 1 {
        for x in -spawn_height..spawn_height + 1 {
            let chunk_x = x + chunk_x;
            let chunk_y = y + chunk_y;
            if let Some(width) = tilemap.width() {
                let width = (width / tilemap.chunk_width()) as i32 / 2;
                if chunk_x < -width || chunk_x > width {
                    continue;
                }
            }
            if let Some(height) = tilemap.height() {
                let height = (height / tilemap.chunk_height()) as i32 / 2;
                if chunk_y < -height || chunk_y > height {
                    continue;
                }
            }

            if let Err(e) = tilemap.spawn_chunk(Point2::new(chunk_x, chunk_y)) {
                warn!("{}", e);
            }
            new_spawned.push(Point2::new(chunk_x, chunk_y));
        }
    }

    let spawned_list = tilemap.spawned_chunks_mut().clone();
    for point in spawned_list.iter() {
        if !new_spawned.contains(&point.into()) {
            if let Err(e) = tilemap.despawn_chunk(point) {
                warn!("{}", e);
            }
        }
    }
}

/// On window size change, the radius of chunks changes if needed.
pub(crate) fn chunk_auto_radius(
    window_resized_events: Res<Events<WindowResized>>,
    mut tilemap_query: Query<(&mut Tilemap, &Transform)>,
    camera_query: Query<(&Camera, &Transform)>,
) {
    let mut window_reader = window_resized_events.get_reader();
    for event in window_reader.iter(&window_resized_events) {
        for (mut tilemap, tilemap_transform) in tilemap_query.iter_mut() {
            let window_width = event.width as u32;
            let window_height = event.height as u32;
            let chunk_px_width = tilemap.chunk_width() * tilemap.tile_width();
            let chunk_px_height = tilemap.chunk_height() * tilemap.tile_height();
            let chunks_wide = (window_width as f32 / chunk_px_width as f32).ceil() as u32 + 1;
            let chunks_high = (window_height as f32 / chunk_px_height as f32).ceil() as u32 + 1;
            let spawn_dimensions = Dimension2::new(chunks_wide, chunks_high);
            tilemap.set_auto_spawn(spawn_dimensions);
            for (_camera, camera_transform) in camera_query.iter() {
                auto_spawn(
                    camera_transform,
                    &tilemap_transform,
                    &mut tilemap,
                    spawn_dimensions,
                );
            }
        }
    }
}

/// Spawns and despawns chunks automatically based on a camera's position.
pub(crate) fn chunk_auto_spawn(
    mut tilemap_query: Query<(&mut Tilemap, &Transform)>,
    camera_query: Query<(&Camera, &Transform), Changed<Transform>>,
) {
    // For the transform, get chunk coord.
    for (mut tilemap, tilemap_transform) in tilemap_query.iter_mut() {
        for (_camera, camera_transform) in camera_query.iter() {
            let spawn_dimensions = if let Some(dimensions) = tilemap.auto_spawn() {
                dimensions
            } else {
                continue;
            };
            auto_spawn(
                camera_transform,
                &tilemap_transform,
                &mut tilemap,
                spawn_dimensions,
            );
        }
    }
}
