use bevy::{
    asset::LoadState, prelude::*, sprite::TextureAtlasBuilder, utils::HashSet, window::WindowMode,
};
use bevy_tilemap::prelude::*;
use rand::Rng;

#[derive(Default, Clone)]
struct TileSpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

#[derive(Default, Clone)]
struct GameState {
    map_loaded: bool,
    spawned: bool,
    collisions: HashSet<(i32, i32)>,
    player: Tile<(i32, i32), Color>,
}

impl GameState {
    fn move_player(&mut self, delta_xy: (i32, i32)) {
        let new_pos = (
            self.player.point.0 + delta_xy.0,
            self.player.point.1 + delta_xy.1,
        );
        if !self.collisions.contains(&new_pos) {
            self.player.point = new_pos;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut tile_sprite_handles: ResMut<TileSpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    tile_sprite_handles.handles = asset_server.load_folder("textures").unwrap();

    commands.spawn(Camera2dComponents::default());
}

fn load(
    mut commands: Commands,
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

        // These are fairly advanced configurations just to quickly showcase
        // them.
        let tilemap = Tilemap::builder()
            .dimensions(1, 1)
            .z_layers(2)
            .texture_atlas(atlas_handle)
            .build()
            .unwrap();

        let tilemap_components = TilemapComponents {
            tilemap,
            transform: Default::default(),
            global_transform: Default::default(),
        };

        commands
            .spawn(tilemap_components)
            .with(Timer::from_seconds(0.075, true));

        sprite_handles.atlas_loaded = true;
    }
}

fn build_random_dungeon(
    mut game_state: ResMut<GameState>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Tilemap>,
) {
    if game_state.map_loaded {
        return;
    }

    for mut map in query.iter_mut() {
        for y in 0..map.height().unwrap() as i32 {
            for x in 0..map.height().unwrap() as i32 {
                map.new_chunk((x, y)).unwrap();
            }
        }

        let width = (map.width().unwrap() * map.chunk_width()) as i32;
        let height = (map.height().unwrap() * map.chunk_height()) as i32;

        // Then we need to find out what the handles were to our textures we are going to use.
        let floor_sprite: Handle<Texture> = asset_server.get_handle("textures/tile_floor.png");
        let wall_sprite: Handle<Texture> = asset_server.get_handle("textures/tile_wall.png");
        let texture_atlas = texture_atlases.get(map.texture_atlas()).unwrap();
        let floor_idx = texture_atlas.get_texture_index(&floor_sprite).unwrap();
        let wall_idx = texture_atlas.get_texture_index(&wall_sprite).unwrap();

        // Now we fill the entire space with floors.
        let mut tiles = Vec::new();
        for y in (-height / 2)..(height / 2) {
            for x in (-width / 2)..(width / 2) {
                // By default tile sets the Z order at 0. Lower means that tile
                // will render lower than others. 0 is the absolute bottom
                // level which is perfect for backgrounds.
                let tile = Tile::new((x, y), floor_idx);
                tiles.push(tile);
            }
        }

        // Then we push in all wall tiles on the X axis.
        for x in (-width / 2)..(width / 2) {
            let tile_a = (x, -height / 2);
            let tile_b = (x, height / 2 - 1);
            tiles.push(Tile::new(tile_a, wall_idx));
            tiles.push(Tile::new(tile_b, wall_idx));
            game_state.collisions.insert(tile_a);
            game_state.collisions.insert(tile_b);
        }

        // Then the wall tiles on the Y axis.
        for y in (-height / 2)..(height / 2) {
            let tile_a = (-width / 2, y);
            let tile_b = (width / 2 - 1, y);
            tiles.push(Tile::new(tile_a, wall_idx));
            tiles.push(Tile::new(tile_b, wall_idx));
            game_state.collisions.insert(tile_a);
            game_state.collisions.insert(tile_b);
        }
        // Lets just generate some random walls to sparsely place around the dungeon!
        let range = (width * height) as usize / 5;
        let mut rng = rand::thread_rng();
        for _ in 0..range {
            let x = rng.gen_range(-width / 2, width / 2);
            let y = rng.gen_range(-width / 2, height / 2);
            let coord = (x, y, 0i32);
            if coord != (0, 0, 0) {
                tiles.push(Tile::new((x, y), wall_idx));
                game_state.collisions.insert((x, y));
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
        map.add_layer_with_kind(LayerKind::Sparse, 1).unwrap();

        // Now lets add in a dwarf friend!
        let dwarf_sprite: Handle<Texture> = asset_server.get_handle("textures/dwarf.png");
        let dwarf_idx = texture_atlas.get_texture_index(&dwarf_sprite).unwrap();
        let dwarf_start_pos = (0, 0);
        // We add in a Z order of 1 to place the tile above the background on Z
        // order 0.
        let dwarf_tile = Tile::with_z_order(dwarf_start_pos, dwarf_idx, 1);
        tiles.push(dwarf_tile);

        game_state.player = dwarf_tile;

        // Now we pass all the tiles to our map.
        map.insert_tiles(tiles).unwrap();

        // Finally we spawn the chunk! In actual use this should be done in a
        // spawn system.
        map.spawn_chunk((0, 0)).unwrap();

        game_state.map_loaded = true;
    }
}

fn character_movement(
    mut game_state: ResMut<GameState>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Tilemap, &mut Timer)>,
) {
    if !game_state.map_loaded {
        return;
    }

    for (mut map, timer) in query.iter_mut() {
        if !timer.finished {
            continue;
        }

        for key in keyboard_input.get_pressed() {
            // First we need to store our very current position.
            let previous_point = game_state.player.point;

            // Of course we need to control where we are going to move our
            // dwarf friend.
            use KeyCode::*;
            match key {
                W | Numpad8 | Up | K => {
                    game_state.move_player((0, 1));
                }
                A | Numpad4 | Left | H => {
                    game_state.move_player((-1, 0));
                }
                S | Numpad2 | Down | J => {
                    game_state.move_player((0, -1));
                }
                D | Numpad6 | Right | L => {
                    game_state.move_player((1, 0));
                }

                Numpad9 | U => game_state.move_player((1, 1)),
                Numpad3 | M => game_state.move_player((1, -1)),
                Numpad1 | N => game_state.move_player((-1, -1)),
                Numpad7 | Y => game_state.move_player((-1, 1)),

                _ => {}
            }

            // If we are standing still, don't do anything.
            if previous_point == game_state.player.point {
                continue;
            }

            // We need to first remove where we were prior.
            map.remove_tile(previous_point, 1).unwrap();
            // We then need to update where we are going!
            map.insert_tile(game_state.player).unwrap();
        }
    }
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Random Tile Dungeon".to_string(),
            width: 1024,
            height: 1024,
            vsync: false,
            resizable: false,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .init_resource::<TileSpriteHandles>()
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(ChunkTilesPlugin::default())
        .add_startup_system(setup.system())
        .add_system(load.system())
        .add_system(build_random_dungeon.system())
        .add_system(character_movement.system())
        .run()
}
