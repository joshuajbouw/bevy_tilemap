use bevy::{asset::LoadState, prelude::*, sprite::TextureAtlasBuilder, window::WindowMode};
use bevy_tilemap::{
    dimensions::{Dimensions2, Dimensions3},
    map::{TileMap, TileMapComponents},
    tile::{Tile, TileSetter},
    ChunkTilesPlugin,
};
use rand::Rng;

#[derive(Default, Clone)]
pub struct TileSpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

#[derive(Default, Clone)]
pub struct MapState {
    map_loaded: bool,
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

        let tile_dimensions = Vec2::new(32., 32.);
        let chunk_dimensions = Vec3::new(32., 32., 0.);
        let tile_map_dimensions = Vec2::new(1., 1.);
        let tile_map = TileMap::new(
            tile_map_dimensions,
            chunk_dimensions,
            tile_dimensions,
            atlas_handle,
        );

        let tile_map_components = TileMapComponents {
            tile_map,
            transform: Default::default(),
            global_transform: Default::default(),
        };

        commands.spawn(tile_map_components);

        sprite_handles.atlas_loaded = true;
    }
}

fn build_random_dungeon(
    mut map_state: ResMut<MapState>,
    mut textures: ResMut<Assets<Texture>>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut TileMap>,
) {
    if map_state.map_loaded {
        return;
    }

    for mut map in query.iter_mut() {
        for y in 0..map.dimensions().x() as i32 {
            for x in 0..map.dimensions().y() as i32 {
                let coord = Vec2::new(x as f32, y as f32);
                map.new_chunk(coord);
            }
        }

        let width = map.dimensions().width() * map.chunk_dimensions().width();
        let height = map.dimensions().height() * map.chunk_dimensions().height();

        // Then we need to find out what the handles were to our textures we are going to use.
        let mut floor_sprite: Handle<Texture> = asset_server.get_handle("textures/tile_floor.png");
        let mut wall_sprite: Handle<Texture> = asset_server.get_handle("textures/tile_wall.png");
        floor_sprite.make_strong(&mut textures);
        wall_sprite.make_strong(&mut textures);

        let texture_atlas = texture_atlases.get(map.texture_atlas()).unwrap();
        let floor_idx = texture_atlas.get_texture_index(&floor_sprite).unwrap();
        let wall_idx = texture_atlas.get_texture_index(&wall_sprite).unwrap();
        let floor_tile = Tile::new(floor_idx);
        let wall_tile = Tile::new(wall_idx);

        // We must use the new handy `TileSetter` tool which is a wrapped `Vec`
        let mut setter = TileSetter::with_capacity((height * width) as usize);
        for y in 0..(height as i32) {
            for x in 0..(width as i32) {
                setter.push(Vec3::new(x as f32, y as f32, 0.), floor_tile);
            }
        }
        // Then we push in all wall tiles on the X axis.
        for x in 0..(width as i32) {
            setter.push(Vec3::new(x as f32, 0., 0.), wall_tile);
            setter.push(Vec3::new(x as f32, height - 1., 0.), wall_tile);
        }
        // Then the wall tiles on the Y axis.
        for y in 0..(height as i32) {
            setter.push(Vec3::new(0., y as f32, 0.), wall_tile);
            setter.push(Vec3::new(width - 1., y as f32, 0.), wall_tile);
        }
        // Lets just generate some random walls to sparsely place around the dungeon!
        let range = (width * height) as usize / 5;
        let mut rng = rand::thread_rng();
        for _ in 0..range {
            let x = rng.gen_range(1, width as i32);
            let y = rng.gen_range(1, height as i32);
            let coord = Vec3::new(x as f32, y as f32, 0.);
            if coord != Vec3::new(width as f32 / 2., height as f32 / 2., 0.) {
                setter.push(Vec3::new(x as f32, y as f32, 0.), wall_tile);
            }
        }

        // Lets do the same as the above, but lets add in a dwarf friend!
        let mut dwarf_sprite: Handle<Texture> = asset_server.get_handle("textures/dwarf.png");
        dwarf_sprite.make_strong(&mut textures);

        let dwarf_idx = texture_atlas.get_texture_index(&dwarf_sprite).unwrap();
        let dwarf_tile = Tile::new(dwarf_idx);
        setter.push(
            Vec3::new(width as f32 / 2., height as f32 / 2., 0.),
            dwarf_tile,
        );

        map.set_tiles(setter).unwrap();

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
