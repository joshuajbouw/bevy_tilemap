use crate::{
    chunk::Chunk,
    coord::{ToCoord3, ToIndex},
    dimensions::{DimensionResult, Dimensions2},
    lib::*,
    tile::{Tile, TileSetter},
};

#[derive(Clone, Copy, PartialEq)]
/// The kinds of errors that can occur for a `[MapError]`.
pub enum ErrorKind {
    OutOfBounds,
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use ErrorKind::*;
        match *self {
            OutOfBounds => write!(f, "out of bounds"),
        }
    }
}

#[derive(Clone, PartialEq)]
/// A MapError indicates that an error with the `[Map]` has occurred.
pub struct MapError(Box<ErrorKind>);

impl Debug for MapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl From<ErrorKind> for MapError {
    fn from(err: ErrorKind) -> MapError {
        MapError::new(err)
    }
}

impl MapError {
    /// Creates a new `MapError`.
    pub fn new(kind: ErrorKind) -> MapError {
        MapError(Box::new(kind))
    }

    /// Returns the underlying error kind `ErrorKind`.
    pub fn kind(&self) -> ErrorKind {
        *self.0
    }
}

/// A map result.
pub type MapResult<T> = Result<T, MapError>;

/// Events that happen on a `Chunk` by index value.
#[derive(Debug)]
pub enum MapEvent<T: Tile, C: Chunk<T>> {
    Created {
        index: usize,
        handle: Handle<C>,
    },
    Refresh {
        handle: Handle<C>,
    },
    Modified {
        handle: Handle<C>,
        setter: TileSetter<T>,
    },
    Despawned {
        handle: Handle<C>,
        entity: Entity,
    },
    Removed {
        index: usize,
        entity: Entity,
    },
}

pub trait TileMap<T: Tile, C: Chunk<T>>:
    'static + Dimensions2 + TypeUuid + Default + Send + Sync
{
    fn set_dimensions(&mut self, dimensions: Vec2);

    fn set_texture_atlas(&mut self, handle: Handle<TextureAtlas>);

    fn texture_atlas_handle(&self) -> &Handle<TextureAtlas>;

    fn handle(&self, index: usize) -> Option<&Handle<C>>;

    fn contains_entity(&self, index: usize) -> bool;

    fn push_chunk_handle(&mut self, index: usize, handle: Option<Handle<C>>);

    fn get_chunk_handle(&self, index: usize) -> Option<&Handle<C>>;

    fn remove_chunk(&mut self, index: usize);

    fn insert_entity(&mut self, index: usize, entity: Entity);

    fn get_entity(&self, index: &usize) -> Option<&Entity>;

    fn events(&self) -> &Events<MapEvent<T, C>>;

    fn send_event(&mut self, event: MapEvent<T, C>);

    fn events_update(&mut self);

    fn events_reader(&mut self) -> EventReader<MapEvent<T, C>>;

    fn add_chunk<I: ToIndex>(&mut self, chunk: C, idx: I, chunks: &mut ResMut<Assets<C>>) {
        let index = idx.to_index(self.dimensions().x(), self.dimensions().y());
        let handle = chunks.add(chunk);
        self.send_event(MapEvent::Created {
            index,
            handle: handle.clone_weak(),
        });
        self.push_chunk_handle(index, Some(handle));
    }

    fn set_chunk<H: Into<HandleId>, I: ToIndex>(
        &mut self,
        handle: H,
        chunk: C,
        idx: I,
        chunks: &mut ResMut<Assets<C>>,
    ) -> DimensionResult<()> {
        let index = idx.to_index(self.dimensions().x(), self.dimensions().y());
        self.check_index(index)?;
        let handle = chunks.set(handle, chunk);
        if self.contains_entity(index) {
            self.send_event(MapEvent::Refresh { handle });
        } else {
            self.send_event(MapEvent::Created { index, handle });
        }
        Ok(())
    }

    fn get_chunk<'a, I: ToIndex>(
        &self,
        idx: I,
        chunks: &'a Assets<C>,
    ) -> DimensionResult<Option<&'a C>> {
        let index = idx.to_index(self.dimensions().x(), self.dimensions().y());
        self.check_index(index)?;
        Ok(self.handle(index).and_then(|handle| chunks.get(handle)))
    }

    fn get_chunk_mut<'a, I: ToIndex>(
        &self,
        idx: I,
        chunks: &'a mut Assets<C>,
    ) -> DimensionResult<Option<&'a mut C>> {
        let index = idx.to_index(self.dimensions().x(), self.dimensions().y());
        self.check_index(index)?;
        Ok(self
            .handle(index)
            .and_then(move |handle| chunks.get_mut(handle)))
    }

    fn chunk_exists<I: ToIndex>(&self, idx: I) -> bool {
        let index = idx.to_index(self.dimensions().x(), self.dimensions().y());
        self.handle(index).is_some()
    }

    fn set_tile<I: ToIndex + ToCoord3>(&mut self, idx: I, tile: T) -> DimensionResult<()> {
        let coord = idx.to_coord3(self.dimensions().x(), self.dimensions().y());
        let chunk_coord = self.tile_coord_to_chunk_coord(coord);
        let chunk_index = chunk_coord.to_index(self.dimensions().x(), self.dimensions().y());
        let handle = self.handle(chunk_index).unwrap().clone_weak();
        let tile_y = coord.y() / C::HEIGHT;
        let map_coord = Vec2::new(
            coord.x() / C::WIDTH,
            self.dimensions().y() - (self.max_y() as f32 - tile_y),
        );
        let x = coord.x() - (map_coord.x() * C::WIDTH);
        let y = C::HEIGHT - 1. - (coord.y() - tile_y * C::HEIGHT);
        let coord = Vec3::new(x, y, coord.z());
        let mut setter = TileSetter::with_capacity(1);
        setter.push(coord, tile);
        self.send_event(MapEvent::Modified { handle, setter });
        Ok(())
    }

    fn set_tiles(&mut self, setter: TileSetter<T>) {
        let mut tiles_map: HashMap<Handle<C>, TileSetter<T>> = HashMap::default();
        for (setter_coord, setter_tile) in setter.iter() {
            let chunk_coord = self.tile_coord_to_chunk_coord(*setter_coord);
            let chunk_index = chunk_coord.to_index(self.dimensions().x(), self.dimensions().y());
            let handle = self.handle(chunk_index).unwrap().clone_weak();
            let tile_y = setter_coord.y() / C::HEIGHT;
            let map_coord = Vec2::new(
                (setter_coord.x() / C::WIDTH).floor(),
                self.max_y() - (self.max_y() as f32 - tile_y),
            );
            let x = setter_coord.x() - (map_coord.x() * C::WIDTH);
            let y = C::X_MAX - (setter_coord.y() - chunk_coord.y() * C::HEIGHT);
            let coord = Vec3::new(x, y, setter_coord.z());
            if let Some(setters) = tiles_map.get_mut(&handle) {
                setters.push(coord, setter_tile.clone());
            } else {
                let mut setter = TileSetter::with_capacity((C::WIDTH * C::HEIGHT) as usize);
                setter.push(coord, setter_tile.clone());
                tiles_map.insert(handle, setter);
            }
        }

        for (handle, setter) in tiles_map {
            self.send_event(MapEvent::Modified { handle, setter })
        }
    }

    /// Returns the center tile of the `Map` as a `Vec2` `Tile` coordinate.
    fn center_tile_coord(&self) -> Vec2 {
        let x = self.dimensions().x() / 2. * C::WIDTH;
        let y = self.dimensions().y() / 2. * C::HEIGHT;
        Vec2::new(x.floor(), y.floor())
    }

    fn tile_coord_to_chunk_coord(&self, coord: Vec3) -> Vec2 {
        let x = (coord.x() / C::WIDTH).floor();
        let y = (coord.y() / C::HEIGHT).floor();
        Vec2::new(x, y)
    }

    /// Takes a translation and calculates the `Tile` coordinate.
    fn translation_to_tile_coord(&self, translation: Vec3) -> Vec2 {
        let center = self.center_tile_coord();
        let x = translation.x() / T::WIDTH as f32 + center.x();
        let y = translation.y() / T::HEIGHT as f32 + center.y();
        Vec2::new(x, y)
    }

    /// Takes a translation and calculates the `Chunk` coordinate.
    fn translation_to_chunk_coord(&self, translation: Vec3) -> Vec2 {
        let center = self.center();
        let x = translation.x() as i32 / (T::WIDTH as i32 * C::HEIGHT as i32) + center.x() as i32;
        let y = translation.y() as i32 / (T::HEIGHT as i32 * C::HEIGHT as i32) + center.y() as i32;
        Vec2::new(x as f32, y as f32)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WorldMap<T: Tile, C: Chunk<T>> {
    dimensions: Vec2,
    #[serde(skip)]
    handles: Vec<Option<Handle<C>>>,
    #[serde(skip)]
    entities: HashMap<usize, Entity>,
    #[serde(skip)]
    events: Events<MapEvent<T, C>>,
    #[serde(skip)]
    texture_atlas: Handle<TextureAtlas>,
}

impl<T: Tile, C: Chunk<T>> Dimensions2 for WorldMap<T, C> {
    fn dimensions(&self) -> Vec2 {
        self.dimensions
    }
}

impl<T: Tile, C: Chunk<T>> TypeUuid for WorldMap<T, C> {
    const TYPE_UUID: Uuid = Uuid::from_u128(109481186966523254410691740507722642628);
}

impl<T: Tile, C: Chunk<T>> TileMap<T, C> for WorldMap<T, C> {
    fn set_dimensions(&mut self, dimensions: Vec2) {
        self.handles = vec![None; (dimensions.x() * dimensions.y()) as usize];
        self.dimensions = dimensions;
    }

    fn set_texture_atlas(&mut self, handle: Handle<TextureAtlas>) {
        self.texture_atlas = handle;
    }

    fn texture_atlas_handle(&self) -> &Handle<TextureAtlas> {
        &self.texture_atlas
    }

    fn handle(&self, index: usize) -> Option<&Handle<C>> {
        self.handles[index].as_ref()
    }

    fn contains_entity(&self, index: usize) -> bool {
        self.entities.contains_key(&index)
    }

    fn push_chunk_handle(&mut self, index: usize, handle: Option<Handle<C>>) {
        self.handles[index] = handle;
    }

    fn get_chunk_handle(&self, index: usize) -> Option<&Handle<C>> {
        self.handles[index].as_ref()
    }

    fn remove_chunk(&mut self, index: usize) {
        self.handles[index] = None;
    }

    fn insert_entity(&mut self, index: usize, entity: Entity) {
        self.entities.insert(index, entity);
    }

    fn get_entity(&self, index: &usize) -> Option<&Entity> {
        self.entities.get(index)
    }

    fn events(&self) -> &Events<MapEvent<T, C>> {
        &self.events
    }

    fn send_event(&mut self, event: MapEvent<T, C>) {
        self.events.send(event);
    }

    fn events_update(&mut self) {
        self.events.update()
    }

    fn events_reader(&mut self) -> EventReader<MapEvent<T, C>> {
        self.events.get_reader()
    }
}

impl<T: Tile, C: Chunk<T>> WorldMap<T, C> {
    pub fn new(dimensions: Vec2, texture_atlas: Handle<TextureAtlas>) -> WorldMap<T, C> {
        let size = (dimensions.x() * dimensions.y()) as usize;
        WorldMap {
            dimensions,
            handles: Vec::with_capacity(size),
            entities: HashMap::default(),
            events: Events::<MapEvent<T, C>>::default(),
            texture_atlas,
        }
    }
}

fn set_tiles<T>(
    tile: &T,
    chunk_texture: &mut Texture,
    sprite_sheet_texture: &Texture,
    sprite_sheet_atlas: &TextureAtlas,
    chunk_rect: Rect,
    chunk_coord: Vec2,
) where
    T: Tile,
{
    let map_texture_size = chunk_texture.size.x() as usize;
    let chunk_format_size = chunk_texture.format.pixel_size();
    let format_size = chunk_texture.format.pixel_size();
    let sprite_idx = {
        if let Some(handle) = tile.texture() {
            sprite_sheet_atlas.get_texture_index(handle).unwrap()
        } else {
            return;
        }
    };
    let sprite_rect = sprite_sheet_atlas.textures[sprite_idx];
    let width = sprite_sheet_texture.size.x() as usize;
    let rect_width = chunk_rect.width() as usize;
    let rect_height = chunk_rect.height() as usize;
    let rect_y = chunk_coord.y() as usize;
    let rect_x = chunk_coord.x() as usize;
    let (sprite_x, mut sprite_y) = (sprite_rect.min.x() as usize, sprite_rect.min.y() as usize);
    for bound_y in rect_y..rect_y + rect_height {
        let begin = (bound_y * map_texture_size + rect_x) * chunk_format_size;
        let end = begin + rect_width * chunk_format_size;
        let sprite_begin = (sprite_y * width + sprite_x) * format_size;
        let sprite_end = sprite_begin + rect_width * format_size;
        chunk_texture.data[begin..end]
            .copy_from_slice(&sprite_sheet_texture.data[sprite_begin..sprite_end]);
        sprite_y += 1;
    }
}

pub fn map_system<T, C, M>(
    mut commands: Commands,
    mut chunks: ResMut<Assets<C>>,
    mut map: ResMut<M>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) where
    T: Tile,
    C: Chunk<T>,
    M: TileMap<T, C>,
{
    map.events_update();
    let mut new_chunks = HashSet::<(usize, Handle<C>)>::default();
    let mut refresh_chunks = HashSet::<Handle<C>>::default();
    let mut modified_chunks = Vec::new();
    let mut despawned_chunks = HashSet::<(Handle<C>, Entity)>::default();
    let mut removed_chunks = HashSet::<(usize, Entity)>::default();
    let mut reader = map.events_reader();
    for event in reader.iter(map.events()) {
        use MapEvent::*;
        match event {
            Created { index, ref handle } => {
                new_chunks.insert((*index, handle.clone_weak()));
            }
            Refresh { ref handle } => {
                refresh_chunks.insert(handle.clone_weak());
            }
            Modified {
                ref handle,
                setter: setters,
            } => {
                modified_chunks.push((handle.clone_weak(), setters.clone()));
            }
            Despawned { ref handle, entity } => {
                despawned_chunks.insert((handle.clone_weak(), *entity));
            }
            Removed { index, entity } => {
                removed_chunks.insert((*index, *entity));
            }
        }
    }

    let sprite_sheet_atlas = texture_atlases.get(map.texture_atlas_handle()).unwrap();
    let sprite_sheet = textures.get(&sprite_sheet_atlas.texture).unwrap().clone();
    for (idx, chunk_handle) in new_chunks.iter() {
        let map_coord = map.decode_coord_unchecked(*idx);
        let map_center = map.center();
        let translation = Vec3::new(
            (map_coord.x() - map_center.x() + 0.5) * T::WIDTH * C::WIDTH,
            (map_coord.y() - map_center.y() + 0.5) * T::HEIGHT * C::HEIGHT,
            1.,
        );
        let chunk = chunks.get_mut(chunk_handle).unwrap();
        let chunk_texture = textures.get_mut(chunk.texture_handle().unwrap()).unwrap();
        for (idx, tile) in chunk.tiles().iter().enumerate() {
            if let Some(tile) = tile {
                let (rect, rect_coord) = {
                    let rect = chunk.textures()[idx];
                    let rect_x = idx % (chunk.dimensions().x() as usize / rect.width() as usize)
                        * rect.width() as usize;
                    let rect_y = idx / (chunk.dimensions().y() as usize / rect.height() as usize)
                        * rect.height() as usize;
                    (rect, Vec2::new(rect_x as f32, rect_y as f32))
                };
                set_tiles(
                    tile,
                    chunk_texture,
                    &sprite_sheet,
                    sprite_sheet_atlas,
                    rect,
                    rect_coord,
                )
            }
        }
        let sprite = {
            SpriteComponents {
                material: materials.add(chunk.texture_handle().unwrap().clone().into()),
                transform: Transform {
                    translation,
                    ..Default::default()
                },
                ..Default::default()
            }
        };
        let entity = commands.spawn(sprite).current_entity().unwrap();
        map.insert_entity(*idx, entity);
    }

    for (chunk_handle, setter) in modified_chunks.iter() {
        let chunk = chunks.get_mut(chunk_handle).unwrap();
        let chunk_texture = textures.get_mut(chunk.texture_handle().unwrap()).unwrap();
        for (setter_coord, setter_tile) in setter.iter() {
            let idx = chunk.encode_coord_unchecked(&setter_coord);
            let (rect, rect_coord) = {
                let rect = chunk.textures()[idx];
                let rect_x = idx % (chunk_texture.size.x() as usize / rect.width() as usize)
                    * rect.width() as usize;
                let rect_y = idx / (chunk_texture.size.y() as usize / rect.height() as usize)
                    * rect.height() as usize;
                (rect, Vec2::new(rect_x as f32, rect_y as f32))
            };
            set_tiles(
                setter_tile,
                chunk_texture,
                &sprite_sheet,
                sprite_sheet_atlas,
                rect,
                rect_coord,
            )
        }
    }

    for (chunk_handle, entity) in despawned_chunks.iter() {
        let chunk = chunks.get_mut(chunk_handle).unwrap();
        chunk.clean();
        commands.despawn(*entity);
    }

    for (index, entity) in removed_chunks.iter() {
        map.remove_chunk(*index);
        commands.despawn(*entity);
    }
}
