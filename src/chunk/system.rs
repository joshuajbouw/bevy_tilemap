use crate::{
    chunk::{entity::Modified, mesh::ChunkMesh},
    lib::*,
    Tilemap,
};

/// The chunk update system that is used to set attributes of the tiles and
/// tints if they need updating.
pub(crate) fn chunk_update(
    mut meshes: ResMut<Assets<Mesh>>,
    map_query: Query<&Tilemap>,
    mut chunk_query: Query<(&Parent, &Point2, &Handle<Mesh>), Changed<Modified>>,
) {
    for (parent, point, mesh_handle) in chunk_query.iter_mut() {
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
        let (indexes, colors) = chunk.tiles_to_renderer_parts(tilemap.chunk_dimensions());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{entity::TilemapBundle, system::tilemap_events, tilemap::TilemapBuilder, Tile};

    #[test]
    fn test_chunk_update() {
        let mut app = AppBuilder::default();
        let app = &mut app
            .add_plugin(ReflectPlugin)
            .add_plugin(CorePlugin)
            .add_plugin(ScheduleRunnerPlugin {})
            .add_plugin(AssetPlugin)
            .add_system_to_stage("update", tilemap_events.system())
            .add_system_to_stage("update", chunk_update.system())
            .add_asset::<Mesh>()
            .app;
        let mut commands = Commands::default();
        commands.set_entity_reserver(app.world.get_entity_reserver());

        let tilemap = TilemapBuilder::new()
            .texture_atlas(Handle::weak(HandleId::random::<TextureAtlas>()))
            .texture_dimensions(32, 32)
            .dimensions(1, 1)
            .chunk_dimensions(5, 5, 1)
            .auto_chunk()
            .z_layers(1)
            .finish()
            .unwrap();
        let tilemap_bundle = TilemapBundle {
            tilemap,
            transform: Default::default(),
            global_transform: Default::default(),
        };

        let _tilemap_entity = commands.spawn(tilemap_bundle).current_entity().unwrap();

        commands.apply(&mut app.world, &mut app.resources);

        let tile_points = vec![
            Point2::new(-2, -2),
            Point2::new(-2, 2),
            Point2::new(2, -2),
            Point2::new(2, 2),
            Point2::new(0, 0),
        ];
        {
            let mut tilemap = app.world.query_mut::<&mut Tilemap>().next().unwrap();
            for tile_point in &tile_points {
                tilemap
                    .insert_tile(Tile {
                        point: *tile_point,
                        sprite_order: 0,
                        sprite_index: 1,
                        tint: Color::RED,
                    })
                    .unwrap();
                tilemap.spawn_chunk(Point2::new(0, 0)).unwrap();
            }
        }

        app.update();

        {
            let tilemap = app.world.query_mut::<&mut Tilemap>().next().unwrap();
            let meshes = app.resources.get::<Assets<Mesh>>().unwrap();
            assert_eq!(meshes.len(), 1);
            let (_, mesh) = meshes.iter().next().unwrap();
            let tile_index = mesh
                .attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX)
                .unwrap()
                .get_bytes();
            assert_eq!(tile_index.len(), 5 * 5 * 4 * 4); // chunk * width * f32 size * byte len

            for tile_point in &tile_points {
                let tile_point = *tile_point + Point2::new(2, 2);
                let index = tilemap
                    .chunk_dimensions()
                    .encode_point(tile_point.into())
                    .unwrap()
                    * 4
                    * 4;
                let mut bytes = Vec::with_capacity(4);
                for x in 0..4 {
                    let byte = tile_index.get(index + x).unwrap();
                    bytes.push(*byte);
                }
                assert_eq!(bytes, [0, 0, 128, 63]);
            }

            let tile_colors = mesh
                .attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR)
                .unwrap()
                .get_bytes();
            assert_eq!(tile_colors.len(), 5 * 5 * 4 * 4 * 4); // chunk * width * f32 size * byte len * 4 bytes in a color

            for tile_point in tile_points {
                let tile_point = tile_point + Point2::new(2, 2);
                let index = tilemap
                    .chunk_dimensions()
                    .encode_point(tile_point.into())
                    .unwrap()
                    * 4
                    * 4
                    * 4;
                let mut bytes = Vec::with_capacity(4 * 4);
                for x in 0..(4 * 4) {
                    let byte = tile_colors.get(index + x).unwrap();
                    bytes.push(*byte);
                }
                assert_eq!(
                    bytes,
                    [255, 255, 127, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63]
                );
            }
        }
    }
}
