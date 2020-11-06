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
    /// If the coordinate or index is out of bounds.
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
    /// To be used when a chunk is created.
    Created {
        /// The map index where the chunk needs to be stored.
        index: usize,
        /// The `Handle` of the `Chunk`.
        handle: Handle<C>,
    },
    /// If the chunk needs to be refreshed.
    ///
    /// # Warning
    /// May never be used, and may be removed.
    Refresh {
        /// The `Handle` of the `Chunk`.
        handle: Handle<C>,
    },
    /// If the chunk had been modified.
    Modified {
        /// The `Handle` of the `Chunk`.
        handle: Handle<C>,
        /// The `TileSetter` that is used to set all the tiles.
        setter: TileSetter<T>,
    },
    /// If the chunk needs to be despawned.
    Despawned {
        /// The `Handle` of the `Chunk`.
        handle: Handle<C>,
        /// The `Entity` that needs to be despawned.
        entity: Entity,
    },
    /// If the chunk needs to be removed.
    ///
    /// # Warning
    /// This is destructive action! All data will be dropped and removed.
    Removed {
        /// The map index where the chunk needs to be removed.
        index: usize,
        /// The `Entity` that needs to be despawned.
        entity: Entity,
    },
}

/// Trait methods for a `TileMap`.
///
/// Provides standard methods for a basic `TileMap` which must be used when
/// using the library's systems.
pub trait TileMap<T: Tile, C: Chunk<T>>:
    'static + Dimensions2 + TypeUuid + Default + Send + Sync
{
    /// Sets the dimensions of the `TileMap`.
    fn set_dimensions(&mut self, dimensions: Vec2);

    /// Sets the sprite sheet, or `TextureAtlas` for use in the `TileMap`.
    fn set_texture_atlas(&mut self, handle: Handle<TextureAtlas>);

    /// Returns a reference the `Handle` of the `TextureAtlas`.
    fn texture_atlas_handle(&self) -> &Handle<TextureAtlas>;

    /// Gets the chunk handle at an index position, if it exists.
    fn get_chunk_handle(&self, index: usize) -> Option<&Handle<C>>;

    /// Returns a bool if the entity exists.
    fn contains_entity(&self, index: usize) -> bool;

    /// Pushes a chunk handle to an index position.
    ///
    /// Do **not** use this with out
    /// storing the `Chunk` as an asset. Preferably, use `add_chunk` instead
    /// which is the correct way to store a `Chunk`.
    fn push_chunk_handle(&mut self, index: usize, handle: Option<Handle<C>>);

    /// Removes a chunk at an index position.
    fn remove_chunk_handle(&mut self, index: usize);

    /// Inserts an `[Entity]` at an index position.
    fn insert_entity(&mut self, index: usize, entity: Entity);

    /// Gets an `[Entity]` at an index position, if it exists.
    fn get_entity(&self, index: &usize) -> Option<&Entity>;

    /// Returns the `[Events]` for the `MapEvent`s.
    fn events(&self) -> &Events<MapEvent<T, C>>;

    /// "Sends" an event by writing it to the current event buffer.
    /// `[EventReader]`s can then read the event.
    fn send_event(&mut self, event: MapEvent<T, C>);

    /// Swaps the event buffers and clears the oldest event buffer. In general,
    /// this should be called once per frame/update.
    fn events_update(&mut self);

    /// Returns the `[EventReader]` containing all `MapEvent`s.
    fn events_reader(&mut self) -> EventReader<MapEvent<T, C>>;

    /// Adds a `Chunk`, creates a handle and stores it at a coordinate position.
    ///
    /// This is the correct way to add a `Chunk`.
    fn add_chunk<I: ToIndex>(&mut self, chunk: C, v: I, chunks: &mut Assets<C>) {
        let index = v.to_index(self.dimensions().x(), self.dimensions().y());
        let handle = chunks.add(chunk);
        self.send_event(MapEvent::Created {
            index,
            handle: handle.clone_weak(),
        });
        self.push_chunk_handle(index, Some(handle));
    }

    /// Sets a `Chunk` with a custom handle at a coordinate position.
    ///
    /// If a `Chunk` already exists, it'll refresh it. If not, it'll create a
    /// new one.
    ///
    /// # Errors
    /// Returns an error if the coordinate is out of bounds.
    fn set_chunk<H: Into<HandleId>, I: ToIndex>(
        &mut self,
        handle: H,
        chunk: C,
        v: I,
        chunks: &mut ResMut<Assets<C>>,
    ) -> DimensionResult<()> {
        let index = v.to_index(self.dimensions().x(), self.dimensions().y());
        self.check_index(index)?;
        let handle = chunks.set(handle, chunk);
        if self.contains_entity(index) {
            self.send_event(MapEvent::Refresh { handle });
        } else {
            self.send_event(MapEvent::Created { index, handle });
        }
        Ok(())
    }

    /// Gets a reference to a `Chunk` from `Chunk` assets and checks if the request is inbounds.
    ///
    /// # Errors
    /// Returns an error if the coordinate is out of bounds.
    fn get_chunk<'a, I: ToIndex>(
        &self,
        v: I,
        chunks: &'a Assets<C>,
    ) -> DimensionResult<Option<&'a C>> {
        let index = v.to_index(self.dimensions().x(), self.dimensions().y());
        self.check_index(index)?;
        Ok(self
            .get_chunk_handle(index)
            .and_then(|handle| chunks.get(handle)))
    }

    /// Gets a mutable reference to a `Chunk` from `Chunk` assets and checks if the request is
    /// inbounds.
    ///
    /// # Errors
    /// Returns an error if the coordinate is out of bounds.
    fn get_chunk_mut<'a, I: ToIndex>(
        &self,
        v: I,
        chunks: &'a mut Assets<C>,
    ) -> DimensionResult<Option<&'a mut C>> {
        let index = v.to_index(self.dimensions().x(), self.dimensions().y());
        self.check_index(index)?;
        Ok(self
            .get_chunk_handle(index)
            .and_then(move |handle| chunks.get_mut(handle)))
    }

    /// Checks if a chunk exists at a coordinate position.
    fn chunk_exists<I: ToIndex>(&self, v: I) -> bool {
        let index = v.to_index(self.dimensions().x(), self.dimensions().y());
        self.get_chunk_handle(index).is_some()
    }

    /// Sets a single tile at a coordinate position and checks if it the request is inbounds.
    ///
    /// # Errors
    /// Returns an error if the coordinate is out of bounds.
    fn set_tile<I: ToIndex + ToCoord3>(&mut self, v: I, tile: T) -> DimensionResult<()> {
        let coord = v.to_coord3(self.dimensions().x(), self.dimensions().y());
        let chunk_coord = self.tile_coord_to_chunk_coord(coord);
        let chunk_index = chunk_coord.to_index(self.dimensions().x(), self.dimensions().y());
        let handle = self.get_chunk_handle(chunk_index).unwrap().clone_weak();
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

    /// Sets many tiles using a `TileSetter`.
    fn set_tiles(&mut self, setter: TileSetter<T>) {
        let mut tiles_map: HashMap<Handle<C>, TileSetter<T>> = HashMap::default();
        for (setter_coord, setter_tile) in setter.iter() {
            let chunk_coord = self.tile_coord_to_chunk_coord(*setter_coord);
            let chunk_index = chunk_coord.to_index(self.dimensions().x(), self.dimensions().y());
            let handle = self.get_chunk_handle(chunk_index).unwrap().clone_weak();
            let tile_y = setter_coord.y() / C::HEIGHT;
            let map_coord = Vec2::new(
                (setter_coord.x() / C::WIDTH).floor(),
                self.max_y() - (self.max_y() as f32 - tile_y),
            );
            let x = setter_coord.x() - (map_coord.x() * C::WIDTH);
            let y = C::X_MAX - (setter_coord.y() - chunk_coord.y() * C::HEIGHT);
            let coord = Vec3::new(x, y, setter_coord.z());
            if let Some(setters) = tiles_map.get_mut(&handle) {
                setters.push_stack(coord, setter_tile.clone());
            } else {
                let mut setter = TileSetter::with_capacity((C::WIDTH * C::HEIGHT) as usize);
                setter.push_stack(coord, setter_tile.clone());
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

    /// Takes a tile coordinate and changes it into a chunk coordinate.
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

/// A basic implementation of the `TileMap` trait.
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

    fn get_chunk_handle(&self, index: usize) -> Option<&Handle<C>> {
        self.handles[index].as_ref()
    }

    fn contains_entity(&self, index: usize) -> bool {
        self.entities.contains_key(&index)
    }

    fn push_chunk_handle(&mut self, index: usize, handle: Option<Handle<C>>) {
        self.handles[index] = handle;
    }

    fn remove_chunk_handle(&mut self, index: usize) {
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
    /// Returns a new WorldMap with the types `Tile` and `Chunk`.
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

fn sprite_coord<T: Tile>(tile: &T, sprite_sheet_atlas: &TextureAtlas) -> Option<(usize, usize)> {
    let sprite_idx = {
        if let Some(handle) = tile.texture() {
            sprite_sheet_atlas.get_texture_index(handle).unwrap()
        } else {
            return None;
        }
    };
    let sprite_rect = sprite_sheet_atlas.textures[sprite_idx];
    Some((sprite_rect.min.x() as usize, sprite_rect.min.y() as usize))
}

fn set_tiles<T>(
    tiles: &[T],
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

    let mut positions = Vec::with_capacity(tiles.len());
    for tile in tiles.iter() {
        if let Some(coord) = sprite_coord(tile, sprite_sheet_atlas) {
            positions.push(coord);
        }
    }
    positions.shrink_to_fit();
    // We need this bool here to prevent borrow issues in the loop.
    let is_single = positions.len() == 1;

    let width = sprite_sheet_texture.size.x() as usize;
    let rect_width = chunk_rect.width() as usize;
    let rect_height = chunk_rect.height() as usize;
    let rect_y = chunk_coord.y() as usize;
    let rect_x = chunk_coord.x() as usize;
    for (sprite_x, mut sprite_y) in positions.iter_mut() {
        for bound_y in rect_y..rect_y + rect_height {
            let begin = (bound_y * map_texture_size + rect_x) * chunk_format_size;
            let end = begin + rect_width * chunk_format_size;
            let sprite_begin = (sprite_y * width + *sprite_x) * format_size;
            let sprite_end = sprite_begin + rect_width * format_size;
            if is_single {
                chunk_texture.data[begin..end]
                    .copy_from_slice(&sprite_sheet_texture.data[sprite_begin..sprite_end]);
            } else {
                let data = &mut chunk_texture.data[begin..end];
                let sprite_data = &sprite_sheet_texture.data[sprite_begin..sprite_end];
                for x in 0..((sprite_end - sprite_begin) / format_size) {
                    let inner_begin = x * format_size;
                    let inner_end = inner_begin + format_size;
                    let inner_data = &sprite_data[inner_begin..inner_end];
                    if inner_data[3] != 0 {
                        data[inner_begin..inner_end]
                            .copy_from_slice(&sprite_data[inner_begin..inner_end])
                    }
                }
            }
            sprite_y += 1;
        }
    }
}

struct MapSystemContext<'a, T: Tile, C: Chunk<T>, M: TileMap<T, C>> {
    commands: Mutex<Commands>,
    chunks: Mutex<ResMut<'a, Assets<C>>>,
    map: Mutex<ResMut<'a, M>>,
    materials: Mutex<ResMut<'a, Assets<ColorMaterial>>>,
    textures: Mutex<ResMut<'a, Assets<Texture>>>,
    sprite_sheet: Mutex<Texture>,
    tile_kind: PhantomData<T>,
}

/// The event handling system for the `TileMap` which takes the types `Tile`, `Chunk`, and `TileMap`.
pub fn map_system<T, C, M>(
    commands: Commands,
    chunks: ResMut<Assets<C>>,
    mut map: ResMut<M>,
    materials: ResMut<Assets<ColorMaterial>>,
    textures: ResMut<Assets<Texture>>,
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

    let pool = TaskPoolBuilder::new()
        .thread_name("map_system".to_string())
        .build();
    let sprite_sheet_atlas = texture_atlases.get(map.texture_atlas_handle()).unwrap();
    let sprite_sheet = textures.get(&sprite_sheet_atlas.texture).unwrap().clone();
    let context = Arc::new(MapSystemContext {
        commands: Mutex::new(commands),
        chunks: Mutex::new(chunks),
        map: Mutex::new(map),
        materials: Mutex::new(materials),
        textures: Mutex::new(textures),
        sprite_sheet: Mutex::new(sprite_sheet),
        tile_kind: Default::default(),
    });
    pool.scope(|scope| {
        for (idx, chunk_handle) in new_chunks.iter() {
            let context = context.clone();
            scope.spawn(async move {
                let context = context.clone();
                let textures = context.textures.lock().unwrap();
                let chunks = context.chunks.lock().unwrap();
                let sprite_sheet_mutex = context.sprite_sheet.lock().unwrap();
                let sprite_sheet = sprite_sheet_mutex.clone();
                let chunk = chunks.get(chunk_handle).unwrap();
                let chunk_tiles = chunk.tiles().clone();
                let chunk_textures = Vec::from(chunk.textures());
                let chunk_texture_handle = chunk.texture_handle().unwrap().clone_weak();
                // clone the texture.
                let mut chunk_texture = textures
                    .get(chunk_texture_handle.clone_weak())
                    .unwrap()
                    .clone();
                // drop all so that other threads can use them.
                ::std::mem::drop(chunks);
                ::std::mem::drop(textures);
                ::std::mem::drop(sprite_sheet_mutex);

                let chunk_texture_size = chunk_texture.size;
                let map_coord = context.map.lock().unwrap().decode_coord_unchecked(*idx);
                let map_center = context.map.lock().unwrap().center();
                for (idx, tile) in chunk_tiles.iter().enumerate() {
                    if let Some(tile) = tile {
                        let (rect, rect_coord) = {
                            let rect = chunk_textures[idx];
                            let rect_x = idx
                                % (chunk_texture_size.x() as usize / rect.width() as usize)
                                * rect.width() as usize;
                            let rect_y = idx
                                / (chunk_texture_size.y() as usize / rect.height() as usize)
                                * rect.height() as usize;
                            (rect, Vec2::new(rect_x as f32, rect_y as f32))
                        };
                        set_tiles(
                            tile,
                            &mut chunk_texture,
                            &sprite_sheet,
                            sprite_sheet_atlas,
                            rect,
                            rect_coord,
                        )
                    }
                }
                let translation = Vec3::new(
                    (map_coord.x() - map_center.x() + 0.5) * T::WIDTH * C::WIDTH,
                    (map_coord.y() - map_center.y() + 0.5) * T::HEIGHT * C::HEIGHT,
                    1.,
                );

                let mut textures = context.textures.lock().unwrap();
                textures.set(chunk_texture_handle.clone_weak(), chunk_texture);
                ::std::mem::drop(textures);

                let sprite = {
                    SpriteComponents {
                        material: context
                            .materials
                            .lock()
                            .unwrap()
                            .add(chunk_texture_handle.into()),
                        transform: Transform {
                            translation,
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                };
                let entity = context
                    .commands
                    .lock()
                    .unwrap()
                    .spawn(sprite)
                    .current_entity()
                    .unwrap();
                context.map.lock().unwrap().insert_entity(*idx, entity);
            });
        }
    });

    pool.scope(|scope| {
        for (chunk_handle, setter) in modified_chunks.iter() {
            let context = context.clone();
            scope.spawn(async move {
                let context = context.clone();
                let mut chunks = context.chunks.lock().unwrap();
                let mut textures = context.textures.lock().unwrap();
                let sprite_sheet_mutex = context.sprite_sheet.lock().unwrap();
                let sprite_sheet = sprite_sheet_mutex.clone();
                let chunk = chunks.get_mut(chunk_handle).unwrap();
                let chunk_texture_handle = chunk.texture_handle().unwrap().clone_weak();
                let chunk_textures = Vec::from(chunk.textures());
                let chunk_dimensions = chunk.dimensions();
                let mut chunk_texture = textures.get_mut(&chunk_texture_handle).unwrap().clone();
                ::std::mem::drop(chunks);
                ::std::mem::drop(textures);
                ::std::mem::drop(sprite_sheet_mutex);

                for (setter_coord, setter_tile) in setter.iter() {
                    let idx = setter_coord.to_index(chunk_dimensions.x(), chunk_dimensions.y());
                    let (rect, rect_coord) = {
                        let rect = chunk_textures[idx];
                        let rect_x = idx
                            % (chunk_texture.size.x() as usize / rect.width() as usize)
                            * rect.width() as usize;
                        let rect_y = idx
                            / (chunk_texture.size.y() as usize / rect.height() as usize)
                            * rect.height() as usize;
                        (rect, Vec2::new(rect_x as f32, rect_y as f32))
                    };
                    set_tiles(
                        setter_tile,
                        &mut chunk_texture,
                        &sprite_sheet,
                        sprite_sheet_atlas,
                        rect,
                        rect_coord,
                    );
                }

                let mut textures = context.textures.lock().unwrap();
                textures.set(chunk_texture_handle.clone_weak(), chunk_texture);
                ::std::mem::drop(textures);
            });
        }
    });

    pool.scope(|scope| {
        for (chunk_handle, entity) in despawned_chunks.iter() {
            let context = context.clone();
            scope.spawn(async move {
                let context = context.clone();
                let mut chunks = context.chunks.lock().unwrap();
                let mut commands = context.commands.lock().unwrap();
                let chunk = chunks.get_mut(chunk_handle).unwrap();
                chunk.clean();
                commands.despawn(*entity);
            });
        }
    });

    pool.scope(|scope| {
        for (index, entity) in removed_chunks.iter() {
            let context = context.clone();
            scope.spawn(async move {
                let context = context.clone();
                let mut map = context.map.lock().unwrap();
                let mut commands = context.commands.lock().unwrap();
                map.remove_chunk_handle(*index);
                commands.despawn(*entity);
            });
        }
    });
}
