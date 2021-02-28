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
    mut tilemap_query: Query<(Entity, &mut Tilemap)>,
    mut modified_query: Query<&mut Modified>,
) {
    for (map_entity, mut tilemap) in tilemap_query.iter_mut() {
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

        let capacity = spawned_chunks.len();
        for point in spawned_chunks.into_iter() {
            if tilemap.spawned_chunks().contains(&(point.x, point.y)) {
                continue;
            } else {
                tilemap.spawned_chunks_mut().insert((point.x, point.y));
            }

            let layers_len = tilemap.layers().len();
            let chunk_dimensions = tilemap.chunk_dimensions();
            let tile_dimensions = tilemap.tile_dimensions();
            let texture_atlas = tilemap.texture_atlas().clone_weak();
            let pipeline_handle = tilemap.topology().to_pipeline_handle();
            let topology = tilemap.topology();
            let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
                chunk
            } else {
                warn!("Can not get chunk at {}, skipping", &point);
                continue;
            };
            let mut entities = Vec::with_capacity(capacity);
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

            use GridTopology::*;
            let translation_x = match topology {
                HexX | HexEvenCols | HexOddCols => {
                    (((chunk.point().x * tile_dimensions.width as i32) as f32 * 0.75) as i32
                        * chunk_dimensions.width as i32) as f32
                }
                HexY => {
                    (chunk.point().x * tile_dimensions.width as i32 * chunk_dimensions.width as i32)
                        as f32
                        + (chunk.point().y as f32 * chunk_dimensions.height as f32 * 0.5)
                            * tile_dimensions.width as f32
                }
                Square | HexEvenRows | HexOddRows => {
                    (chunk.point().x * tile_dimensions.width as i32 * chunk_dimensions.width as i32)
                        as f32
                }
            };
            let translation_y = match topology {
                HexX => {
                    (chunk.point().y
                        * tile_dimensions.height as i32
                        * chunk_dimensions.height as i32) as f32
                        + (chunk.point().x as f32 * chunk_dimensions.width as f32 * 0.5)
                            * tile_dimensions.height as f32
                }
                HexY | HexEvenRows | HexOddRows => {
                    (((chunk.point().y * tile_dimensions.height as i32) as f32 * 0.75) as i32
                        * chunk_dimensions.height as i32) as f32
                }
                Square | HexEvenCols | HexOddCols => {
                    (chunk.point().y
                        * tile_dimensions.height as i32
                        * chunk_dimensions.height as i32) as f32
                }
            };
            // TODO: set translation Z from somewhere else.
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
                error!("Chunk entity does not exist unexpectedly, can not run the tilemap system");
                return;
            };

            info!("Chunk {} spawned", point);

            chunk.set_entity(entity);
            entities.push(entity);
            // }
            commands.push_children(map_entity, &entities);
        }

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

        for point in despawned_chunks.into_iter() {
            let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
                chunk
            } else {
                warn!("Can not get chunk at {}, skipping", &point);
                continue;
            };
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
}
