use bevy::{
    asset::LoadState, ecs::bevy_utils::HashMapExt, prelude::*, sprite::TextureAtlasBuilder,
    window::WindowMode,
};
use bevy_tilemap::prelude::*;
use rand::Rng;

#[derive(Default, Clone)]
pub struct TileSpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

#[derive(Default, Clone)]
pub struct MapState {
    map_loaded: bool,
    spawned: bool,
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

        let tilemap = Tilemap::builder()
            .dimensions(1, 1)
            .chunk_dimensions(32, 32)
            .tile_dimensions(32, 32)
            .z_layers(2)
            .texture_atlas(atlas_handle)
            .build()
            .unwrap();

        let tilemap_components = TilemapComponents {
            tilemap,
            transform: Default::default(),
            global_transform: Default::default(),
        };

        commands.spawn(tilemap_components);

        sprite_handles.atlas_loaded = true;
    }
}

fn build_random_dungeon(
    mut map_state: ResMut<MapState>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Tilemap>,
) {
    if map_state.map_loaded {
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
        let floor_tile = Tile::new(floor_idx);
        let wall_tile = Tile::new(wall_idx);

        // We must use the new handy `Tiles` tool which is a wrapped `Vec`
        let mut tiles = Tiles::with_capacity((height * width) as usize);
        for y in 0..height {
            for x in 0..width {
                tiles.insert((x, y, 0), floor_tile);
            }
        }
        // Then we push in all wall tiles on the X axis.
        for x in 0..width {
            tiles.insert((x, 0, 0), wall_tile);
            tiles.insert((x, height - 1, 0), wall_tile);
        }
        // Then the wall tiles on the Y axis.
        for y in 0..height {
            tiles.insert((0, y, 0), wall_tile);
            tiles.insert((width - 1, y, 0), wall_tile);
        }
        // Lets just generate some random walls to sparsely place around the dungeon!
        let range = (width * height) as usize / 5;
        let mut rng = rand::thread_rng();
        for _ in 0..range {
            let x = rng.gen_range(1, width as i32);
            let y = rng.gen_range(1, height as i32);
            let coord = (x, y, 0i32);
            if coord != (width / 2, height / 2, 0) {
                tiles.insert((x, y, 0), wall_tile);
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
        let dwarf_tile = Tile::new(dwarf_idx);
        tiles.insert(
            (width / 2, height / 2, 1), // Do note that we are pushing him onto z_layer 1 now!
            dwarf_tile,
        );

        // Now we pass all the tiles to our map.
        map.set_tiles(tiles).unwrap();

        // Finally we spawn the chunk! In actual use this should be done in a
        // spawn system.
        map.spawn_chunk((0, 0)).unwrap();

        map_state.map_loaded = true;
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
        .init_resource::<MapState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(ChunkTilesPlugin::default())
        .add_startup_system(setup.system())
        .add_system(load.system())
        .add_system(build_random_dungeon.system())
        .run()
}
