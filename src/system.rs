//! The tilemap systems.

#[cfg(feature = "bevy_rapier3d")]
use crate::{chunk::Chunk, TilemapLayer};
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

/// Spawns collisions based on given arguments.
///
/// This is a bit messy and has quite a few inputs but, quite a few parts had
/// to be cloned. There very likely is a better way to clean this up.
#[cfg(feature = "bevy_rapier3d")]
fn spawn_collisions(
    commands: &mut Commands,
    layers: &[Option<TilemapLayer>],
    point: Point3,
    sprite_order: usize,
    chunk: &mut Chunk,
    chunk_dimensions: Dimension3,
    tile_dimensions: Dimension2,
    transform: &Transform,
    physics_tile_width: f32,
    physics_tile_height: f32,
    // physics_tile_depth: f32,
) {
    // Don't continue if there is no layer.
    if let Some(layer_opt) = layers.get(sprite_order) {
        match layer_opt {
            Some(layer) => {
                if layer.interaction_groups.0 == 0 {
                    return;
                }
            }
            None => return,
        }
    }
    // Don't continue if there is no entity.
    let entity = match chunk.get_entity() {
        Some(e) => e,
        None => return,
    };
    // Don't continue if there already is a collision there.
    let index = chunk_dimensions.encode_point_unchecked(point);
    if chunk.get_collision_entity(point.z as usize, index).is_some() {
        return;
    }
    let mut collision_entities = Vec::new();
    if let Some(indices) = chunk.get_tile_indices(point.z as usize, sprite_order) {
        for index in &indices {
            let point = match chunk_dimensions.decode_point(*index) {
                Ok(p) => p,
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
            };
            // Adjust half a width and height back.
            let mut x = (point.x - chunk_dimensions.width as i32 / 2) as f32;
            let mut y = (point.y - chunk_dimensions.height as i32 / 2) as f32;
            let mut z = (point.z - chunk_dimensions.depth as i32 / 2) as f32;
            // Adjust by chunk position
            x += chunk.point().x as f32
                * chunk_dimensions.width as f32
                * tile_dimensions.width as f32;
            y += chunk.point().y as f32
                * chunk_dimensions.height as f32
                * tile_dimensions.height as f32;
            // Add tilemap's translation
            x += transform.translation.x;
            y += transform.translation.y;

            if chunk_dimensions.width % 2 == 0 {
                x += 0.5;
            }
            if chunk_dimensions.height % 2 == 0 {
                y += 0.5;
            }

            let collision_groups = layers
                .get(sprite_order)
                .and_then(|layer_opt| layer_opt.and_then(|layer| Some(layer.interaction_groups)));
            if let Some(collision_groups) = collision_groups {
                if collision_groups.with_mask(0).0 != 0 {
                    let mut collider = ColliderBuilder::cuboid(
                        physics_tile_width / 2.0,
                        physics_tile_height / 2.0,
                        // physics_tile_depth / 2.0,
                    );

                    collider = collider.collision_groups(collision_groups);

                    let entity = if let Some(entity) = commands
                        .spawn((
                            RigidBodyBuilder::new_static() // TODO: double check point.z
                                .translation(x * physics_tile_width, y * physics_tile_height, point.z as f32 * physics_tile_depth),
                            collider,
                        ))
                        .current_entity()
                    {
                        entity
                    } else {
                        error!("Collider entity does not exist unexpectedly, can not run the tilemap system");
                        return;
                    };

                    collision_entities.push(entity);
                }
            }
        }
        for (index, entity) in indices.iter().zip(&collision_entities) {
            chunk.insert_collision_entity(point.z as usize, *index, *entity);
        }
        commands.push_children(entity, &collision_entities);
    }
}

/// The event handling system for collisions. Namely spawning and despawning.
///
/// Depending on if a collision needs to be created or not, given a variety of
/// conditions, collisions are spawned or despawned accordingly.
#[cfg(feature = "bevy_rapier3d")]
pub(crate) fn tilemap_collision_events(
    commands: &mut Commands,
    mut tilemap_query: Query<(&mut Tilemap, &Transform)>,
) {
    for (mut tilemap, transform) in tilemap_query.iter_mut() {
        if tilemap.topology() != GridTopology::Square {
            error!("collision physics are not supported on hex tiles yet");
            continue;
        }
        tilemap.collision_events_update();
        let mut spawned_chunks = Vec::new();
        let mut reader = tilemap.chunk_events().get_reader();
        for event in reader.iter(&tilemap.chunk_events()) {
            use crate::TilemapChunkEvent::*;
            match event {
                Spawned { ref point } => {
                    spawned_chunks.push(*point);
                }
                _ => continue,
            };
        }

        for point in spawned_chunks.into_iter() {
            let layers = tilemap.layers();
            let layers_len = tilemap.layers().len();
            let chunk_dimensions = tilemap.chunk_dimensions();
            let tile_dimensions = tilemap.tile_dimensions();
            let physics_tile_width = tile_dimensions.width as f32 / tilemap.physics_scale();
            let physics_tile_height = tile_dimensions.height as f32 / tilemap.physics_scale();
            // let physics_tile_depth = tile_dimensions.depth as f32 / tilemap.physics_scale();
            let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&point) {
                chunk
            } else {
                warn!("Can not get chunk at {}, skipping", &point);
                continue;
            };
            for sprite_order in 0..layers_len {
                spawn_collisions(
                    commands,
                    &layers,
                    point,
                    sprite_order,
                    chunk,
                    chunk_dimensions,
                    tile_dimensions,
                    transform,
                    physics_tile_width,
                    physics_tile_height,
                    physics_tile_depth,
                );
            }
        }

        let mut spawned_collisions = Vec::new();
        let mut despawned_collisions = Vec::new();
        let mut reader = tilemap.collision_events().get_reader();
        for event in reader.iter(&tilemap.collision_events()) {
            use crate::event::TilemapCollisionEvent::*;
            match event {
                Spawned {
                    ref chunk_point,
                    ref tiles,
                } => {
                    spawned_collisions.push((*chunk_point, tiles.clone()));
                }
                Despawned {
                    ref chunk_point,
                    ref tiles,
                } => {
                    despawned_collisions.push((*chunk_point, tiles.clone()));
                }
            };
        }

        for (chunk_point, tiles) in spawned_collisions.into_iter() {
            let layers = tilemap.layers();
            let chunk_dimensions = tilemap.chunk_dimensions();
            let tile_dimensions = tilemap.tile_dimensions();
            let physics_tile_width = tile_dimensions.width as f32 / tilemap.physics_scale();
            let physics_tile_height = tile_dimensions.height as f32 / tilemap.physics_scale();
            let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&chunk_point) {
                chunk
            } else {
                warn!("Can not get chunk at {}, skipping", &chunk_point);
                continue;
            };
            for tile in tiles {
                spawn_collisions(
                    commands,
                    &layers,
                    tile.point,
                    tile.sprite_order,
                    chunk,
                    chunk_dimensions,
                    tile_dimensions,
                    transform,
                    physics_tile_width,
                    physics_tile_height,
                );
            }
        }

        for (chunk_point, tiles) in despawned_collisions.into_iter() {
            let chunk_dimensions = tilemap.chunk_dimensions();
            let chunk = if let Some(chunk) = tilemap.chunks_mut().get_mut(&chunk_point) {
                chunk
            } else {
                warn!("Can not get chunk at {}, skipping", &chunk_point);
                continue;
            };
            for tile in tiles {
                let index = chunk_dimensions.encode_point_unchecked(tile.point);
                let collision_entity = chunk.get_collision_entity(index);
                if let Some(entity) = collision_entity {
                    commands.despawn(entity);
                    info!(
                        "Tile {} on z order {} collision entity despawned",
                        tile.point, tile.sprite_order
                    );
                }
            }
        }
    }
}
