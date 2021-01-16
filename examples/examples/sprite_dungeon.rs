#![allow(clippy::all)]
use bevy::{
    asset::LoadState,
    prelude::*,
    render::camera::Camera,
    sprite::{TextureAtlas, TextureAtlasBuilder},
    utils::HashSet,
    window::WindowMode,
};
pub(crate) use bevy_rapier2d::{
    rapier::{
        dynamics::RigidBodyBuilder,
        geometry::{ColliderBuilder, InteractionGroups},
    },
    render::RapierRenderPlugin,
    physics::RapierPhysicsPlugin,
};
use bevy_tilemap::prelude::*;
use rand::Rng;

const CHUNK_WIDTH: u32 = 16;
const CHUNK_HEIGHT: u32 = 16;
const TILEMAP_WIDTH: i32 = 16;
const TILEMAP_HEIGHT: i32 = 16;

#[derive(Default, Clone)]
struct TileSpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

#[derive(Default, Copy, Clone, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Default)]
struct Render {
    sprite_index: usize,
    z_order: usize,
}

#[derive(Default)]
struct Player {}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    position: Position,
    render: Render,
}

#[derive(Default, Clone)]
struct GameState {
    map_loaded: bool,
    spawned: bool,
}

fn setup(mut tile_sprite_handles: ResMut<TileSpriteHandles>, asset_server: Res<AssetServer>) {
    tile_sprite_handles.handles = asset_server.load_folder("textures").unwrap();
}

fn load(
    commands: &mut Commands,
    mut sprite_handles: ResMut<TileSpriteHandles>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    asset_server: Res<AssetServer>,
) {
    if sprite_handles.atlas_loaded {
        return;
    }

    // Lets load all our textures from our folder!
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    if let LoadState::Loaded =
        asset_server.get_group_load_state(sprite_handles.handles.iter().map(|handle| handle.id))
    {
        for handle in sprite_handles.handles.iter() {
            let texture = textures.get(handle).unwrap();
            texture_atlas_builder.add_texture(handle.clone_weak().typed::<Texture>(), &texture);
        }

        let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
        let atlas_handle = texture_atlases.add(texture_atlas);

        let background_layer = TilemapLayer {
            kind: LayerKind::Dense,
            ..Default::default()
        };
        let wall_interactions =
            InteractionGroups::new(0b0000_0000_0000_0001, 0b0000_0000_0000_0010);
        let wall_layer = TilemapLayer {
            kind: LayerKind::Sparse,
            // interaction_groups: wall_interactions,
            ..Default::default()
        };
        // These are fairly advanced configurations just to quickly showcase
        // them.
        let tilemap = Tilemap::builder()
            .dimensions(TILEMAP_WIDTH as u32, TILEMAP_HEIGHT as u32)
            .chunk_dimensions(CHUNK_WIDTH, CHUNK_HEIGHT)
            .auto_chunk()
            .auto_spawn(2, 2)
            .z_layers(2)
            .add_layer(background_layer, 0)
            .add_layer(wall_layer, 1)
            .texture_atlas(atlas_handle)
            .finish()
            .unwrap();

        let tilemap_components = TilemapBundle {
            tilemap,
            transform: Default::default(),
            global_transform: Default::default(),
        };

        commands.spawn(Camera2dBundle::default());
        commands
            .spawn(tilemap_components)
            .with(Timer::from_seconds(0.075, true));

        sprite_handles.atlas_loaded = true;
    }
}

fn build_random_dungeon(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Tilemap>,
) {
    if game_state.map_loaded {
        return;
    }

    for mut map in query.iter_mut() {
        // Then we need to find out what the handles were to our textures we are going to use.
        let floor_sprite: Handle<Texture> = asset_server.get_handle("textures/square-floor.png");
        let wall_sprite: Handle<Texture> = asset_server.get_handle("textures/square-wall.png");
        let texture_atlas = texture_atlases.get(map.texture_atlas()).unwrap();
        let floor_idx = texture_atlas.get_texture_index(&floor_sprite).unwrap();
        let wall_idx = texture_atlas.get_texture_index(&wall_sprite).unwrap();

        // Now we fill the entire space with floors.
        let mut tiles = Vec::new();
        for y in 0..TILEMAP_HEIGHT {
            for x in 0..TILEMAP_WIDTH {
                let y = y - TILEMAP_HEIGHT / 2;
                let x = x - TILEMAP_WIDTH / 2;
                // By default tile sets the Z order at 0. Lower means that tile
                // will render lower than others. 0 is the absolute bottom
                // level which is perfect for backgrounds.
                let tile = Tile::new((x, y), floor_idx);
                tiles.push(tile);
            }
        }

        for x in 0..TILEMAP_WIDTH {
            let x = x - TILEMAP_WIDTH / 2;
            let tile_a = (x, -TILEMAP_HEIGHT / 2);
            let tile_b = (x, TILEMAP_HEIGHT / 2 - 1);
            tiles.push(Tile::new(tile_a, wall_idx));
            tiles.push(Tile::new(tile_b, wall_idx));
        }

        // Then the wall tiles on the Y axis.
        for y in 0..TILEMAP_HEIGHT {
            let y = y - TILEMAP_HEIGHT / 2;
            let tile_a = (-TILEMAP_WIDTH / 2, y);
            let tile_b = (TILEMAP_WIDTH / 2 - 1, y);
            tiles.push(Tile::new(tile_a, wall_idx));
            tiles.push(Tile::new(tile_b, wall_idx));
        }

        // Lets just generate some random walls to sparsely place around the dungeon!
        let range = (TILEMAP_WIDTH * TILEMAP_HEIGHT) as usize / 5;
        let mut rng = rand::thread_rng();
        for _ in 0..range {
            let x = rng.gen_range((-TILEMAP_WIDTH / 2)..(TILEMAP_WIDTH / 2));
            let y = rng.gen_range((-TILEMAP_HEIGHT / 2)..(TILEMAP_HEIGHT / 2));
            let coord = (x, y, 0i32);
            if coord != (0, 0, 0) {
                tiles.push(Tile::new((x, y), wall_idx));
            }
        }

        // The above should give us a neat little randomized dungeon! However,
        // we are missing a hero! First, we need to add a layer. We must make
        // this layer `Sparse` else we will lose efficiency with our data!
        //
        // You might've noticed that we didn't create a layer for z_layer 0 but
        // yet it still works and exists. By default if a layer doesn't exist
        // and tiles need to be written there then a Dense layer is created
        // automatically.
        // map.add_layer_with_kind(LayerKind::Sparse, 1).unwrap();

        // // Now lets add in a dwarf friend!
        let dwarf_sprite: Handle<Texture> = asset_server.get_handle("textures/square-dwarf.png");
        let dwarf_sprite_index = texture_atlas.get_texture_index(&dwarf_sprite).unwrap();
        let translation = Vec3::new(16.0, 16.0, 3.0);
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: map.texture_atlas().clone_weak(),
                sprite: TextureAtlasSprite {
                    index: dwarf_sprite_index as u32,
                    ..Default::default()
                },
                transform: Transform::from_translation(translation),
                ..Default::default()
            })
            .with(Player {})
            .with(RigidBodyBuilder::new_dynamic())
            .with(ColliderBuilder::cuboid(32.0, 32.0).translation(0.0, 0.0).collision_groups(InteractionGroups::new(0b0000_0000_0000_0010, 0b0000_0000_0001)));

        // Now we pass all the tiles to our map.
        map.insert_tiles(tiles).unwrap();

        game_state.map_loaded = true;
    }
}

fn character_movement(
    mut game_state: ResMut<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut map_query: Query<(&mut Tilemap, &mut Timer)>,
    mut player_query: Query<(&mut Transform, &Player)>,
    mut camera_query: Query<(&Camera, &mut Transform)>,
) {
    if !game_state.map_loaded {
        return;
    }

    for (mut map, mut timer) in map_query.iter_mut() {
        for (mut position, _player) in player_query.iter_mut() {
            for key in keyboard_input.get_pressed() {
                for (_camera, mut camera_transform) in camera_query.iter_mut() {
                    let move_step = 0.5;
                    // Of course we need to control where we are going to move our
                    // dwarf friend.
                    use KeyCode::*;
                    match key {
                        W => {
                            position.translation.y += move_step;
                        }
                        A => {
                            position.translation.x -= move_step;
                        }
                        S => {
                            position.translation.y -= move_step;
                        }
                        D => {
                            position.translation.x += move_step;
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Physics Dungeon".to_string(),
            width: 1024.,
            height: 1024.,
            vsync: false,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .init_resource::<TileSpriteHandles>()
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapDefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(setup.system())
        .add_system(load.system())
        .add_system(build_random_dungeon.system())
        .add_system(character_movement.system())
        .run()
}
