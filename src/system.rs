//! The tilemap systems.

use crate::{
    chunk::{
        entity::{ChunkBundle, Modified},
        mesh::ChunkMesh,
        render::GridTopology,
        Chunk, LayerKind,
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
    tilemap_entity: Entity,
    tilemap_visible: &Visible,
    meshes: &mut Assets<Mesh>,
    tilemap: &mut Tilemap,
    spawned_chunks: Vec<Point2>,
) {
    let capacity = spawned_chunks.len();
    let mut entities = Vec::with_capacity(capacity);
    for point in spawned_chunks.into_iter() {
        if tilemap.spawned_chunks().contains(&(point.x, point.y)) {
            continue;
        } else {
            tilemap.spawned_chunks_mut().insert((point.x, point.y));
        }

        let chunk_dimensions = tilemap.chunk_dimensions();
        let texture_dimensions = tilemap.texture_dimensions();
        let texture_atlas = tilemap.texture_atlas().clone_weak();
        let pipeline_handle = tilemap.topology().to_pipeline_handle();
        let chunk_mesh = tilemap.chunk_mesh().clone();
        let topology = tilemap.topology();
        let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
            chunk
        } else {
            // NOTE: should this instead create a chunk if it doesn't exist yet?
            warn!("Can not get chunk at {}, possible bug report me", &point);
            continue;
        };
        let mut mesh = Mesh::from(&chunk_mesh);
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
        let entity = commands
            .spawn()
            .insert_bundle(ChunkBundle {
                point,
                texture_atlas: texture_atlas.clone_weak(),
                mesh: mesh_handle.clone_weak(),
                transform: Transform::from_translation(translation),
                render_pipelines: RenderPipelines::from_pipelines(vec![pipeline]),
                draw: Default::default(),
                visible: tilemap_visible.clone(),
                main_pass: MainPass,
                global_transform: Default::default(),
                modified: Default::default(),
            })
            .id();

        info!("Chunk {} spawned", point);

        chunk.set_entity(entity);
        entities.push(entity);
    }
    commands.entity(tilemap_entity).push_children(&entities);
}

/// Handles all modified chunks and flags them.
fn handle_modified_chunks(
    modified_query: &mut Query<&mut Modified>,
    tilemap: &mut Tilemap,
    modified_chunks: Vec<Point2>,
) {
    for point in modified_chunks.into_iter() {
        let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
            chunk
        } else {
            warn!("Can not get chunk at {}, skipping", &point);
            continue;
        };
        if let Some(chunk_entity) = chunk.get_entity() {
            if let Ok(mut modified) = modified_query.get_mut(chunk_entity) {
                modified.0 += 1;
            }
        } else {
            continue;
        };
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
                commands.entity(e).despawn_recursive();
                info!("Chunk {} despawned", point);
            }
            None => {
                continue;
            }
        }
    }
}

/// Recalculates a mesh.
fn recalculate_mesh(
    meshes: &mut Assets<Mesh>,
    mesh: &Handle<Mesh>,
    chunk: &Chunk,
    chunk_mesh: &ChunkMesh,
    chunk_dimensions: Dimension3,
) {
    let mesh = match meshes.get_mut(mesh) {
        None => {
            error!("tried to get mesh for chunk but failed, this is a bug");
            return;
        }
        Some(m) => m,
    };
    let (indexes, colors) = chunk.tiles_to_renderer_parts(chunk_dimensions);
    mesh.set_indices(Some(Indices::U32(chunk_mesh.indices.clone())));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, chunk_mesh.vertices.clone());
    mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes);
    mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors);
}

/// Adds a sprite layer to all chunks and recalculates the mesh.
fn handle_add_sprite_layers(
    meshes: &mut Assets<Mesh>,
    tilemap: &mut Tilemap,
    add_sprite_layers: Vec<(LayerKind, usize)>,
) {
    let chunk_dimensions = tilemap.chunk_dimensions();
    let chunk_mesh = tilemap.chunk_mesh().clone();
    for chunk in tilemap.chunks_mut().values_mut() {
        for (kind, sprite_layer) in &add_sprite_layers {
            chunk.add_sprite_layer(&kind, *sprite_layer, chunk_dimensions);
            if let Some(mesh) = chunk.mesh() {
                recalculate_mesh(meshes, mesh, chunk, &chunk_mesh, chunk_dimensions);
            }
        }
    }
}

/// Removes a sprite layer from all chunks and recalculates the mesh if needed.
fn handle_remove_sprite_layers(
    meshes: &mut Assets<Mesh>,
    tilemap: &mut Tilemap,
    remove_sprite_layers: Vec<usize>,
) {
    let chunk_dimensions = tilemap.chunk_dimensions();
    let chunk_mesh = tilemap.chunk_mesh().clone();
    for sprite_layer in remove_sprite_layers {
        for chunk in tilemap.chunks_mut().values_mut() {
            chunk.remove_sprite_layer(sprite_layer);
            if let Some(mesh) = chunk.mesh() {
                recalculate_mesh(meshes, mesh, chunk, &chunk_mesh, chunk_dimensions);
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
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tilemap_query: Query<(Entity, &mut Tilemap, &Visible)>,
    mut modified_query: Query<&mut Modified>,
) {
    for (tilemap_entity, mut tilemap, tilemap_visible) in tilemap_query.iter_mut() {
        tilemap.chunk_events_update();
        let mut reader = tilemap.chunk_events().get_reader();

        let mut modified_chunks = Vec::new();
        let mut spawned_chunks = Vec::new();
        let mut despawned_chunks = Vec::new();
        let mut add_sprite_layers = Vec::new();
        let mut remove_sprite_layers = Vec::new();
        for event in reader.iter(&tilemap.chunk_events()) {
            use crate::TilemapChunkEvent::*;
            match event {
                Modified { ref point } => {
                    modified_chunks.push(*point);
                }
                Spawned { ref point } => {
                    spawned_chunks.push(*point);
                }
                Despawned { ref point } => {
                    despawned_chunks.push(*point);
                }
                AddLayer {
                    ref layer_kind,
                    ref sprite_layer,
                } => {
                    add_sprite_layers.push((*layer_kind, *sprite_layer));
                }
                RemoveLayer { ref sprite_layer } => {
                    remove_sprite_layers.push(*sprite_layer);
                }
            }
        }

        if !spawned_chunks.is_empty() {
            handle_spawned_chunks(
                &mut commands,
                tilemap_entity,
                tilemap_visible,
                &mut meshes,
                &mut tilemap,
                spawned_chunks,
            );
        }

        if !modified_chunks.is_empty() {
            handle_modified_chunks(&mut modified_query, &mut tilemap, modified_chunks);
        }

        if !despawned_chunks.is_empty() {
            handle_despawned_chunks(&mut commands, &mut tilemap, despawned_chunks);
        }

        if !add_sprite_layers.is_empty() {
            handle_add_sprite_layers(&mut meshes, &mut tilemap, add_sprite_layers);
        }

        if !remove_sprite_layers.is_empty() {
            handle_remove_sprite_layers(&mut meshes, &mut tilemap, remove_sprite_layers);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{entity::TilemapBundle, tilemap::TilemapBuilder};

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
            // .add_plugin(ReflectPlugin)
            .add_plugin(CorePlugin)
            .add_plugin(ScheduleRunnerPlugin {})
            .add_plugin(AssetPlugin)
            .add_stage("update", SystemStage::parallel())
            .add_system_to_stage("update", tilemap_events.system())
            .add_asset::<Mesh>()
            .app;
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &app.world);

        let tilemap = new_tilemap();
        let tilemap_bundle = TilemapBundle {
            tilemap,
            visible: Visible {
                is_visible: true,
                is_transparent: true,
            },
            transform: Default::default(),
            global_transform: Default::default(),
        };

        let tilemap_entity = commands.spawn().insert_bundle(tilemap_bundle).id();

        command_queue.apply(&mut app.world);

        {
            let mut tilemap = app
                .world
                .query::<&mut Tilemap>()
                .iter_mut(&mut app.world)
                .next()
                .unwrap();
            tilemap.insert_chunk(Point2::new(0, 0)).unwrap();
            tilemap.insert_chunk(Point2::new(1, 1)).unwrap();
            tilemap.insert_chunk(Point2::new(-1, -1)).unwrap();
            tilemap.spawn_chunk(Point2::new(0, 0)).unwrap();
            tilemap.spawn_chunk(Point2::new(1, 1)).unwrap();
            tilemap.spawn_chunk(Point2::new(-1, -1)).unwrap();
        }

        app.update();

        {
            let meshes = &mut app.world.get_resource_mut::<Assets<Mesh>>().unwrap();
            assert_eq!(meshes.len(), 3);
        }

        {
            let tilemap_children = app.world.get::<Children>(tilemap_entity).unwrap().len();
            assert_eq!(tilemap_children, 3);
        }

        {
            let mut tilemap = app
                .world
                .query::<&mut Tilemap>()
                .iter_mut(&mut app.world)
                .next()
                .unwrap();
            tilemap.modify_chunk(Point2::new(1, 1));
        }

        app.update();

        // This test isn't working as intended as it seems that query_filtered
        // just might not actually be working. This should be explored.
        {
            // let mut modified_query = app.world.query_filtered::<&Point2, Changed<Modified>>();
            let mut found = false;
            for modified in app.world.query::<&Modified>().iter(&app.world) {
                if modified.0 == 1 {
                    found = true;
                }
            }
            assert!(found);
        }

        // then despawn one, both entities and meshes should be -1
        {
            let mut tilemap = app
                .world
                .query::<&mut Tilemap>()
                .iter_mut(&mut app.world)
                .next()
                .unwrap();
            tilemap.despawn_chunk(Point2::new(-1, -1)).unwrap();
        }

        app.update();

        let chunks_count = app
            .world
            .query::<(Entity, &Modified)>()
            .iter(&app.world)
            .count();
        assert_eq!(chunks_count, 2);

        // Need to double update to kick the GC.
        app.update();
        app.update();

        let meshes = app.world.get_resource::<Assets<Mesh>>().unwrap();
        assert_eq!(meshes.len(), 2);
    }
}
