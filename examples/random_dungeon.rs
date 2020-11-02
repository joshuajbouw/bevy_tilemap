use bevy::{
    asset::LoadState, prelude::*, render::texture::TextureFormat, sprite::TextureAtlasBuilder,
    window::WindowMode,
};
use bevy_chunk_tiles::{
    chunk::{Chunk, WorldChunk},
    dimensions::Dimensions2,
    map::{TileMap, WorldMap},
    tile::{Tile, TileSetter},
    ChunkTilesPlugin,
};
use rand::Rng;

#[derive(Debug, Default, Clone)]
pub struct WorldTile {
    texture: Handle<Texture>,
    coord: Vec2,
}

impl Tile for WorldTile {
    const WIDTH: f32 = 32.0;
    const HEIGHT: f32 = 32.0;

    fn texture(&self) -> Option<&Handle<Texture>> {
        Some(&self.texture)
    }

    fn coord(&self) -> Option<Vec2> {
        Some(self.coord)
    }
}

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
    mut map: ResMut<WorldMap<WorldTile, WorldChunk<WorldTile>>>,
    mut tile_sprite_handles: ResMut<TileSpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    tile_sprite_handles.handles = asset_server.load_folder("textures").unwrap();
    map.set_dimensions(Vec2::new(3., 3.));

    commands.spawn(Camera2dComponents::default());
}

fn load_atlas(
    mut map: ResMut<WorldMap<WorldTile, WorldChunk<WorldTile>>>,
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
        map.set_texture_atlas(atlas_handle);
        sprite_handles.atlas_loaded = true;
    }
}

fn build_random_dungeon(
    mut map: ResMut<WorldMap<WorldTile, WorldChunk<WorldTile>>>,
    mut map_state: ResMut<MapState>,
    mut chunks: ResMut<Assets<WorldChunk<WorldTile>>>,
    mut textures: ResMut<Assets<Texture>>,
    asset_server: Res<AssetServer>,
) {
    if map_state.map_loaded {
        return;
    }

    let width = map.dimensions().x() * WorldChunk::<WorldTile>::WIDTH;
    let height = map.dimensions().y() * WorldChunk::<WorldTile>::HEIGHT;

    // First we need to create the chunks adn add them
    for y in 0..map.dimensions().x() as i32 {
        for x in 0..map.dimensions().y() as i32 {
            let coord = Vec2::new(x as f32, y as f32);
            let mut chunk = WorldChunk::default();
            let texture = Texture::new_fill(
                chunk.pixel_dimensions(),
                &[255, 0, 0, 255],
                TextureFormat::Rgba8UnormSrgb,
            );
            let texture_handle = textures.add(texture);
            chunk.set_texture_handle(Some(texture_handle));
            map.add_chunk(chunk, coord, &mut chunks);
        }
    }

    // Then we need to find out what the handles were to our textures we are going to use.
    let mut floor_sprite = asset_server.get_handle("textures/tile_floor.png");
    let mut wall_sprite: Handle<Texture> = asset_server.get_handle("textures/tile_wall.png");
    floor_sprite.make_strong(&mut textures);
    wall_sprite.make_strong(&mut textures);
    let floor_tile = WorldTile {
        texture: floor_sprite,
        coord: Vec2::new(0., 0.),
    };
    let wall_tile = WorldTile {
        texture: wall_sprite,
        coord: Vec2::new(0., 0.),
    };
    // We must use the new handy `TileSetter` tool which is a wrapped `Vec`.
    let mut setter = TileSetter::with_capacity((height * width) as usize);
    // Now we push in all floor tiles.
    for y in 0..(height as i32) {
        for x in 0..(width as i32) {
            setter.push(Vec3::new(x as f32, y as f32, 0.), floor_tile.clone());
        }
    }
    // Then we push in all wall tiles on the X axis.
    for x in 0..(width as i32) {
        setter.push(Vec3::new(x as f32, 0., 0.), wall_tile.clone());
        setter.push(Vec3::new(x as f32, height - 1., 0.), wall_tile.clone());
    }
    // Then the wall tiles on the Y axis.
    for y in 0..(height as i32) {
        setter.push(Vec3::new(0., y as f32, 0.), wall_tile.clone());
        setter.push(Vec3::new(width - 1., y as f32, 0.), wall_tile.clone());
    }
    // Lets just generate some random walls to sparsely place around the dungeon!
    let range = (width * height) as usize / 5;
    let mut rng = rand::thread_rng();
    for _ in 0..range {
        let x = rng.gen_range(1, width as i32);
        let y = rng.gen_range(1, height as i32);
        let coord = Vec3::new(x as f32, y as f32, 0.);
        if coord != Vec3::new(width as f32 / 2., height as f32 / 2., 0.) {
            setter.push(Vec3::new(x as f32, y as f32, 0.), wall_tile.clone());
        }
    }
    // Lets do the same as the above, but lets add in a dwarven friend!
    let mut dwarf_sprite: Handle<Texture> = asset_server.get_handle("textures/dwarf_idle.png");
    dwarf_sprite.make_strong(&mut textures);
    let dwarf_tile = WorldTile {
        texture: dwarf_sprite,
        coord: Vec2::new(0., 0.),
    };
    setter.push(
        Vec3::new(width as f32 / 2., height as f32 / 2., 0.),
        dwarf_tile,
    );

    map.set_tiles(setter);
    map_state.map_loaded = true;
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
        .add_default_plugins()
        .add_plugin(ChunkTilesPlugin::<
            WorldTile,
            WorldChunk<WorldTile>,
            WorldMap<WorldTile, WorldChunk<WorldTile>>,
        >::default())
        .add_startup_system(setup.system())
        .add_system(load_atlas.system())
        .add_system(build_random_dungeon.system())
        .run()
}
