//! The tilemap systems.

use crate::{
    chunk::{
        entity::{ChunkBundle, Modified},
        mesh::ChunkMesh,
        render::GridTopology,
    },
    lib::*,
    Tilemap,
};

/// Takes a grid topology and returns altered translation coordinates.
// TODO: set translation Z from somewhere else.
fn topology_translation(
    topology: GridTopology,
    chunk_point: Point2,
    chunk_dimensions: Dimension3,
    texture_dimensions: Dimension2,
) -> (f32, f32) {
    use GridTopology::*;
    let translation_x = match topology {
        HexX | HexEvenCols | HexOddCols => {
            (((chunk_point.x * texture_dimensions.width as i32) as f32 * 0.75) as i32
                * chunk_dimensions.width as i32) as f32
        }
        HexY => {
            (chunk_point.x * texture_dimensions.width as i32 * chunk_dimensions.width as i32) as f32
                + (chunk_point.y as f32 * chunk_dimensions.height as f32 * 0.5)
                    * texture_dimensions.width as f32
        }
        Square | HexEvenRows | HexOddRows => {
            (chunk_point.x * texture_dimensions.width as i32 * chunk_dimensions.width as i32) as f32
        }
    };
    let translation_y = match topology {
        HexX => {
            (chunk_point.y * texture_dimensions.height as i32 * chunk_dimensions.height as i32)
                as f32
                + (chunk_point.x as f32 * chunk_dimensions.width as f32 * 0.5)
                    * texture_dimensions.height as f32
        }
        HexY | HexEvenRows | HexOddRows => {
            (((chunk_point.y * texture_dimensions.height as i32) as f32 * 0.75) as i32
                * chunk_dimensions.height as i32) as f32
        }
        Square | HexEvenCols | HexOddCols => {
            (chunk_point.y * texture_dimensions.height as i32 * chunk_dimensions.height as i32)
                as f32
        }
    };

    (translation_x, translation_y)
}

/// Handles all newly spawned chunks and attempts to spawn them.
fn handle_spawned_chunks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    tilemap: &mut Tilemap,
    spawned_chunks: Vec<Point2>,
) -> Vec<Entity> {
    let capacity = spawned_chunks.len();
    let mut entities = Vec::with_capacity(capacity);
    for point in spawned_chunks.into_iter() {
        if tilemap.spawned_chunks().contains(&(point.x, point.y)) {
            continue;
        } else {
            tilemap.spawned_chunks_mut().insert((point.x, point.y));
        }

        let layers_len = tilemap.layers().len();
        let chunk_dimensions = tilemap.chunk_dimensions();
        let texture_dimensions = tilemap.texture_dimensions();
        let texture_atlas = tilemap.texture_atlas().clone_weak();
        let pipeline_handle = tilemap.topology().to_pipeline_handle();
        let topology = tilemap.topology();
        let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
            chunk
        } else {
            // NOTE: should this instead create a chunk if it doesn't exist yet?
            warn!("Can not get chunk at {}, possible bug report me", &point);
            continue;
        };
        let mut mesh = Mesh::from(&ChunkMesh::new(
            chunk_dimensions,
            layers_len as u32,
            Vec2::new(0., 0.), // TODO: put actual value here
        ));
        let (indexes, colors) = chunk.tiles_to_renderer_parts(chunk_dimensions);
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes);
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors);
        let mesh_handle = meshes.add(mesh);
        chunk.set_mesh(mesh_handle.clone());

        let (translation_x, translation_y) = topology_translation(
            topology,
            chunk.point(),
            chunk_dimensions,
            texture_dimensions,
        );
        let translation = Vec3::new(translation_x, translation_y, 1.0);
        let pipeline = RenderPipeline::new(pipeline_handle.clone_weak().typed());
        let entity = if let Some(entity) = commands
            .spawn(ChunkBundle {
                point,
                texture_atlas: texture_atlas.clone_weak(),
                mesh: mesh_handle.clone_weak(),
                transform: Transform::from_translation(translation),
                render_pipelines: RenderPipelines::from_pipelines(vec![pipeline]),
                draw: Default::default(),
                visible: Visible {
                    // TODO: this would be nice as a config parameter to make
                    // RapierRenderPlugin's output visible.
                    is_visible: true,
                    is_transparent: true,
                },
                main_pass: MainPass,
                global_transform: Default::default(),
                modified: Default::default(),
            })
            .current_entity()
        {
            entity
        } else {
            error!("Chunk entity does not exist unexpectedly, report this");
            continue;
        };

        info!("Chunk {} spawned", point);

        chunk.set_entity(entity);
        entities.push(entity);
    }
    entities
}

/// Handles all modified chunks and flags them.
fn handle_modified_chunks(
    modified_query: &mut Query<&mut Modified>,
    tilemap: &mut Tilemap,
    modified_chunks: Vec<HashMap<usize, Point2>>,
) {
    for layers in modified_chunks.into_iter() {
        for (_layer, point) in layers.into_iter() {
            let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
                chunk
            } else {
                warn!("Can not get chunk at {}, skipping", &point);
                continue;
            };
            if let Some(entity) = chunk.get_entity() {
                let mut count = if let Ok(count) = modified_query.get_mut(entity) {
                    count
                } else {
                    warn!(
                        "Can not increment modified count for chunk {}, skipping",
                        point
                    );
                    continue;
                };
                count.0 += 1;
            } else {
                warn!("Can not take entity from chunk {}, skipping", point);
                continue;
            };
        }
    }
}

/// Handles all despawned chunks and attempts to despawn them.
fn handle_despawned_chunks(
    commands: &mut Commands,
    tilemap: &mut Tilemap,
    despawned_chunks: Vec<Point2>,
) {
    for point in despawned_chunks.into_iter() {
        let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
            chunk
        } else {
            warn!("Can not get chunk at {}, skipping", &point);
            continue;
        };

        chunk.take_mesh();

        match chunk.take_entity() {
            Some(e) => {
                commands.despawn_recursive(e);
                info!("Chunk {} despawned", point);
            }
            None => {
                warn!("Can not take entity from chunk {}, skipping", point);
                continue;
            }
        }
    }
}

/// The event handling system for the tilemap.
///
/// There are a few things that happen in this function which are outlined in
/// order of operation here. It was done in this order that made the most sense
/// at the time of creation.
///
/// 1. Spawn chunks
/// 1. Modify chunks
/// 1. Despawn chunks
pub(crate) fn tilemap_events(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tilemap_query: Query<(Entity, &mut Tilemap, &Visible)>,
    mut modified_query: Query<&mut Modified>,
) {
    for (map_entity, mut tilemap, tilemap_visible) in tilemap_query.iter_mut() {
        tilemap.chunk_events_update();
        let mut modified_chunks = Vec::new();
        let mut spawned_chunks = Vec::new();
        let mut despawned_chunks = Vec::new();
        let mut reader = tilemap.chunk_events().get_reader();
        for event in reader.iter(&tilemap.chunk_events()) {
            use crate::TilemapChunkEvent::*;
            match event {
                Modified { ref layers } => {
                    modified_chunks.push(layers.clone());
                }
                Spawned { ref point } => {
                    spawned_chunks.push(*point);
                }
                Despawned { ref point } => {
                    despawned_chunks.push(*point);
                }
            }
        }

        let entities = handle_spawned_chunks(commands, &mut meshes, &mut tilemap, spawned_chunks);
        commands.push_children(map_entity, &entities);

        handle_modified_chunks(&mut modified_query, &mut tilemap, modified_chunks);

        handle_despawned_chunks(commands, &mut tilemap, despawned_chunks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tilemap::TilemapBuilder;

    fn new_tilemap() -> Tilemap {
        TilemapBuilder::new()
            .texture_atlas(Handle::weak(HandleId::random::<TextureAtlas>()))
            .texture_dimensions(32, 32)
            .finish()
            .unwrap()
    }

    #[test]
    fn test_topology_translations() {
        let topologies = vec![
            (
                GridTopology::Square,
                vec![
                    (-4096.0, -930.0),
                    (-2048.0, -465.0),
                    (0.0, 0.0),
                    (2048.0, 465.0),
                    (4096.0, 930.0),
                ],
            ),
            (
                GridTopology::HexEvenCols,
                vec![
                    (-3072.0, -930.0),
                    (-1536.0, -465.0),
                    (0.0, 0.0),
                    (1536.0, 465.0),
                    (3072.0, 930.0),
                ],
            ),
            (
                GridTopology::HexEvenRows,
                vec![
                    (-4096.0, -682.0),
                    (-2048.0, -341.0),
                    (0.0, 0.0),
                    (2048.0, 341.0),
                    (4096.0, 682.0),
                ],
            ),
            (
                GridTopology::HexOddCols,
                vec![
                    (-3072.0, -930.0),
                    (-1536.0, -465.0),
                    (0.0, 0.0),
                    (1536.0, 465.0),
                    (3072.0, 930.0),
                ],
            ),
            (
                GridTopology::HexOddRows,
                vec![
                    (-4096.0, -682.0),
                    (-2048.0, -341.0),
                    (0.0, 0.0),
                    (2048.0, 341.0),
                    (4096.0, 682.0),
                ],
            ),
            (
                GridTopology::HexX,
                vec![
                    (-3072.0, -1890.0),
                    (-1536.0, -945.0),
                    (0.0, 0.0),
                    (1536.0, 945.0),
                    (3072.0, 1890.0),
                ],
            ),
            (
                GridTopology::HexY,
                vec![
                    (-5088.0, -682.0),
                    (-2544.0, -341.0),
                    (0.0, 0.0),
                    (2544.0, 341.0),
                    (5088.0, 682.0),
                ],
            ),
        ];
        let chunk_points = vec![
            Point2::new(-2, -2),
            Point2::new(-1, -1),
            Point2::new(0, 0),
            Point2::new(1, 1),
            Point2::new(2, 2),
        ];
        let chunk_dimensions = Dimension3::new(64, 31, 0);
        let texture_dimensions = Dimension2::new(32, 15);

        for (topology, tests) in topologies.into_iter() {
            for (chunk_point, test) in chunk_points.iter().zip(tests) {
                let res = topology_translation(
                    topology,
                    *chunk_point,
                    chunk_dimensions,
                    texture_dimensions,
                );
                assert_eq!(res, test);
            }
        }
    }

    #[test]
    fn insert_and_spawn_chunk() {
        let mut app = AppBuilder::default();
        let app = &mut app
            .add_plugin(ReflectPlugin)
            .add_plugin(CorePlugin)
            .add_plugin(ScheduleRunnerPlugin {})
            .add_plugin(AssetPlugin)
            .add_asset::<Mesh>()
            .app;
        let mut commands = Commands::default();
        commands.set_entity_reserver(app.world.get_entity_reserver());

        let mut tilemap = new_tilemap();
        tilemap.insert_chunk(Point2::new(0, 0)).unwrap();
        tilemap.insert_chunk(Point2::new(1, 1)).unwrap();
        tilemap.insert_chunk(Point2::new(-1, -1)).unwrap();

        {
            let meshes = &mut app.resources.get_mut::<Assets<Mesh>>().unwrap();
            let spawned_chunks = vec![Point2::new(0, 0), Point2::new(1, 1), Point2::new(-1, -1)];
            let entities =
                handle_spawned_chunks(&mut commands, meshes, &mut tilemap, spawned_chunks);

            assert_eq!(entities.len(), 3);
            assert_eq!(meshes.len(), 3);
        }

        commands.apply(&mut app.world, &mut app.resources);
        app.update();
        // then despawn one, both entities and meshes should be -1

        let chunk_points = vec![Point2::new(1, 1)];
        handle_despawned_chunks(&mut commands, &mut tilemap, chunk_points);

        commands.apply(&mut app.world, &mut app.resources);
        app.update();
        app.update();

        let meshes = &app.resources.get::<Assets<Mesh>>().unwrap();
        let chunks = app.world.query::<(Entity, &Modified)>();
        assert_eq!(chunks.count(), 2);
        assert_eq!(meshes.len(), 2);
    }
}

/// Checks for tilemap visibility changes and reflects them on all chunks.
pub fn tilemap_visibility_change(
    tilemap_visible_query: Query<(Entity, &Tilemap)>,
    mut visibles: Query<&mut Visible, Changed<Visible>>,
) {
    for (entity, tilemap) in tilemap_visible_query.iter() {
        let tilemap_visible = if let Ok(visible) = visibles.get_mut(entity) {
            visible.clone()
        } else {
            continue;
        };
        for chunk in tilemap.chunks().values() {
            if let Some(entity) = chunk.get_entity() {
                if let Ok(mut chunk_visible) = visibles.get_mut(entity) {
                    *chunk_visible = tilemap_visible.clone();
                }
            }
        }
    }
}
