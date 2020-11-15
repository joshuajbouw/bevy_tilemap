use crate::{
    chunk::Chunk,
    coord::{ToCoord3, ToIndex},
    dimensions::{DimensionResult, Dimensions2, Dimensions3},
    entity::ChunkComponents,
    lib::*,
    mesh::ChunkMesh,
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
pub enum MapEvent {
    /// To be used when a chunk is created.
    Created {
        /// The map index where the chunk needs to be stored.
        index: usize,
        /// The `Handle` of the `Chunk`.
        handle: Handle<Chunk>,
    },
    /// If the chunk needs to be refreshed.
    ///
    /// # Warning
    /// May never be used, and may be removed.
    Refresh {
        /// The `Handle` of the `Chunk`.
        handle: Handle<Chunk>,
    },
    /// If the chunk had been modified.
    Modified {
        /// The `Handle` of the `Chunk`.
        handle: Handle<Chunk>,
        /// The `TileSetter` that is used to set all the tiles.
        setter: TileSetter,
    },
    /// If the chunk needs to be despawned.
    Despawned {
        /// The `Handle` of the `Chunk`.
        handle: Handle<Chunk>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TileMapDimensions(Vec2);

impl Dimensions2 for TileMapDimensions {
    fn dimensions(&self) -> Vec2 {
        self.0
    }
}

impl TileMapDimensions {
    pub fn new(dimensions: Vec2) -> TileMapDimensions {
        TileMapDimensions(dimensions)
    }
}

impl From<Vec2> for TileMapDimensions {
    fn from(v: Vec2) -> TileMapDimensions {
        TileMapDimensions(v)
    }
}

/// A basic implementation of the `TileMap` trait.
#[derive(Debug, Serialize, Deserialize, RenderResources)]
pub struct TileMap {
    #[render_resources(ignore)]
    dimensions: Vec2,
    chunk_dimensions: Vec3,
    #[render_resources(ignore)]
    tile_dimensions: Vec2,
    #[serde(skip)]
    // Should change to HashSet when merged into bevy
    #[render_resources(ignore)]
    handles: Vec<Option<Handle<Chunk>>>,
    #[serde(skip)]
    #[render_resources(ignore)]
    entities: HashMap<usize, Entity>,
    #[serde(skip)]
    #[render_resources(ignore)]
    events: Events<MapEvent>,
    #[serde(skip)]
    #[render_resources(ignore)]
    texture_atlas: Handle<TextureAtlas>,
}

impl TypeUuid for TileMap {
    const TYPE_UUID: Uuid = Uuid::from_u128(109481186966523254410691740507722642628);
}

impl TileMap {
    /// Returns a new WorldMap with the types `Tile` and `Chunk`.
    pub fn new(
        dimensions: Vec2,
        chunk_dimensions: Vec3,
        tile_dimensions: Vec2,
        texture_atlas: Handle<TextureAtlas>,
    ) -> TileMap {
        let capacity = (dimensions.x() * dimensions.y()) as usize;
        TileMap {
            dimensions,
            chunk_dimensions,
            tile_dimensions,
            handles: Vec::with_capacity(capacity),
            entities: HashMap::default(),
            events: Events::<MapEvent>::default(),
            texture_atlas,
        }
    }

    /// Sets the dimensions of the `TileMap`.
    pub fn set_dimensions(&mut self, dimensions: Vec2) {
        self.handles = vec![None; (dimensions.x() * dimensions.y()) as usize];
        self.dimensions = dimensions;
    }

    /// Sets the sprite sheet, or `TextureAtlas` for use in the `TileMap`.
    pub fn set_texture_atlas(&mut self, handle: Handle<TextureAtlas>) {
        self.texture_atlas = handle;
    }

    /// Returns a reference the `Handle` of the `TextureAtlas`.
    pub fn texture_atlas_handle(&self) -> &Handle<TextureAtlas> {
        &self.texture_atlas
    }

    /// Gets the chunk handle at an index position, if it exists.
    pub fn get_chunk_handle(&self, index: usize) -> Option<&Handle<Chunk>> {
        self.handles[index].as_ref()
    }

    /// Returns a bool if the entity exists.
    pub fn contains_entity(&self, index: usize) -> bool {
        self.entities.contains_key(&index)
    }

    /// Pushes a chunk handle to an index position.
    ///
    /// Do **not** use this with out
    /// storing the `Chunk` as an asset. Preferably, use `add_chunk` instead
    /// which is the correct way to store a `Chunk`.
    pub fn push_chunk_handle(&mut self, index: usize, handle: Option<Handle<Chunk>>) {
        self.handles[index] = handle;
    }

    /// Removes a chunk at an index position.
    pub fn remove_chunk_handle(&mut self, index: usize) {
        self.handles[index] = None;
    }

    /// Inserts an `[Entity]` at an index position.
    pub fn insert_entity(&mut self, index: usize, entity: Entity) {
        self.entities.insert(index, entity);
    }

    /// Gets an `[Entity]` at an index position, if it exists.
    pub fn get_entity(&self, index: &usize) -> Option<&Entity> {
        self.entities.get(index)
    }

    /// Returns the `[Events]` for the `MapEvent`s.
    pub fn events(&self) -> &Events<MapEvent> {
        &self.events
    }

    /// "Sends" an event by writing it to the current event buffer.
    /// `[EventReader]`s can then read the event.
    pub fn send_event(&mut self, event: MapEvent) {
        self.events.send(event);
    }

    /// Swaps the event buffers and clears the oldest event buffer. In general,
    /// this should be called once per frame/update.
    pub fn events_update(&mut self) {
        self.events.update()
    }

    /// Returns the `[EventReader]` containing all `MapEvent`s.
    pub fn events_reader(&mut self) -> EventReader<MapEvent> {
        self.events.get_reader()
    }

    /// Adds a `Chunk`, creates a handle and stores it at a coordinate position.
    ///
    /// This is the correct way to add a `Chunk`.
    pub fn add_chunk<I: ToIndex>(&mut self, chunk: Chunk, v: I, chunks: &mut Assets<Chunk>) {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
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
    pub fn set_chunk<H: Into<HandleId>, I: ToIndex>(
        &mut self,
        handle: H,
        chunk: Chunk,
        v: I,
        chunks: &mut Assets<Chunk>,
    ) -> DimensionResult<()> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.y());
        self.dimensions.check_index(index)?;
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
    pub fn get_chunk<'a, I: ToIndex>(
        &self,
        v: I,
        chunks: &'a Assets<Chunk>,
    ) -> DimensionResult<Option<&'a Chunk>> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
        self.dimensions.check_index(index)?;
        Ok(self
            .get_chunk_handle(index)
            .and_then(|handle| chunks.get(handle)))
    }

    /// Gets a mutable reference to a `Chunk` from `Chunk` assets and checks if the request is
    /// inbounds.
    ///
    /// # Errors
    /// Returns an error if the coordinate is out of bounds.
    pub fn get_chunk_mut<'a, I: ToIndex>(
        &self,
        v: I,
        chunks: &'a mut Assets<Chunk>,
    ) -> DimensionResult<Option<&'a mut Chunk>> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
        self.dimensions.check_index(index)?;
        Ok(self
            .get_chunk_handle(index)
            .and_then(move |handle| chunks.get_mut(handle)))
    }

    /// Checks if a chunk exists at a coordinate position.
    pub fn chunk_exists<I: ToIndex>(&self, v: I) -> bool {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
        self.get_chunk_handle(index).is_some()
    }

    /// Sets a single tile at a coordinate position and checks if it the request is inbounds.
    ///
    /// # Errors
    /// Returns an error if the coordinate is out of bounds.
    pub fn set_tile<I: ToIndex + ToCoord3>(&mut self, v: I, tile: Tile) -> DimensionResult<()> {
        let coord = v.to_coord3(self.dimensions.width(), self.dimensions.height());
        let chunk_coord = self.tile_coord_to_chunk_coord(coord);
        let chunk_index = chunk_coord.to_index(self.dimensions.width(), self.dimensions.height());
        let handle = self.get_chunk_handle(chunk_index).unwrap().clone_weak();
        let tile_y = coord.y() / self.chunk_dimensions.height();
        let map_coord = Vec2::new(
            coord.x() / self.chunk_dimensions.width(),
            self.dimensions.height() - (self.dimensions.max_y() as f32 - tile_y),
        );
        let x = coord.x() - (map_coord.x() * self.chunk_dimensions.width());
        let y =
            self.chunk_dimensions.max_y() - (coord.y() - tile_y * self.chunk_dimensions.height());
        let coord = Vec3::new(x, y, coord.z());
        let mut setter = TileSetter::with_capacity(1);
        setter.push(coord, tile);
        self.send_event(MapEvent::Modified { handle, setter });
        Ok(())
    }

    /// Sets many tiles using a `TileSetter`.
    pub fn set_tiles(&mut self, setter: TileSetter) {
        let mut tiles_map: HashMap<Handle<Chunk>, TileSetter> = HashMap::default();
        for (setter_coord, setter_tile) in setter.iter() {
            let chunk_coord = self.tile_coord_to_chunk_coord(*setter_coord);
            let chunk_index =
                chunk_coord.to_index(self.dimensions.width(), self.dimensions.height());
            let handle = self.get_chunk_handle(chunk_index).unwrap().clone_weak();
            let tile_y = setter_coord.y() / self.chunk_dimensions.height();
            let map_coord = Vec2::new(
                (setter_coord.x() / self.chunk_dimensions.width()).floor(),
                self.dimensions.max_y() - (self.dimensions.max_y() as f32 - tile_y),
            );
            let x = setter_coord.x() - (map_coord.x() * self.chunk_dimensions.width());
            let y = self.chunk_dimensions.max_y()
                - (setter_coord.y() - chunk_coord.y() * self.chunk_dimensions.height());
            let coord = Vec3::new(x, y, setter_coord.z());
            if let Some(setters) = tiles_map.get_mut(&handle) {
                setters.push(coord, *setter_tile);
            } else {
                let mut setter = TileSetter::with_capacity(
                    (self.chunk_dimensions.width() * self.chunk_dimensions.height()) as usize,
                );
                setter.push(coord, *setter_tile);
                tiles_map.insert(handle, setter);
            }
        }

        for (handle, setter) in tiles_map {
            self.send_event(MapEvent::Modified { handle, setter })
        }
    }

    /// Returns the center tile of the `Map` as a `Vec2` `Tile` coordinate.
    pub fn center_tile_coord(&self) -> Vec2 {
        let x = self.dimensions.width() / 2. * self.chunk_dimensions.y();
        let y = self.dimensions.height() / 2. * self.chunk_dimensions.x();
        Vec2::new(x.floor(), y.floor())
    }

    /// Takes a tile coordinate and changes it into a chunk coordinate.
    pub fn tile_coord_to_chunk_coord(&self, coord: Vec3) -> Vec2 {
        let x = (coord.x() / self.chunk_dimensions.y()).floor();
        let y = (coord.y() / self.chunk_dimensions.x()).floor();
        Vec2::new(x, y)
    }

    /// Takes a translation and calculates the `Tile` coordinate.
    pub fn translation_to_tile_coord(&self, translation: Vec3) -> Vec2 {
        let center = self.center_tile_coord();
        let x = translation.x() / self.tile_dimensions.width() as f32 + center.x();
        let y = translation.y() / self.tile_dimensions.height() as f32 + center.y();
        Vec2::new(x, y)
    }

    /// Takes a translation and calculates the `Chunk` coordinate.
    pub fn translation_to_chunk_coord(&self, translation: Vec3) -> Vec2 {
        let center = self.dimensions.center();
        let x = translation.x() as i32
            / (self.tile_dimensions.width() as i32 * self.chunk_dimensions.width() as i32)
            + center.x() as i32;
        let y = translation.y() as i32
            / (self.tile_dimensions.height() as i32 * self.chunk_dimensions.height() as i32)
            + center.y() as i32;
        Vec2::new(x as f32, y as f32)
    }

    pub fn dimensions(&self) -> Vec2 {
        self.dimensions
    }

    pub fn chunk_dimensions(&self) -> Vec3 {
        self.chunk_dimensions
    }

    /// A Chunk's size in pixels.
    pub fn chunk_size(&self) -> Vec2 {
        Vec2::new(
            self.tile_dimensions.width() * self.chunk_dimensions.width(),
            self.tile_dimensions.height() * self.chunk_dimensions.height(),
        )
    }

    pub fn tile_dimensions(&self) -> Vec2 {
        self.tile_dimensions
    }

    /// Takes a `Tile` coordinate and returns its location in the `Map`.
    pub fn chunk_to_world_coord(&self, coord: &Vec3, translation: Vec2) -> Option<Vec3> {
        // takes in translation of tile
        let chunk_x = (translation.x() / self.tile_dimensions.width() / self.dimensions.width())
            + self.dimensions.max_x()
            - 1.;
        let chunk_y = 2.
            - (translation.y() / self.tile_dimensions.height() / self.dimensions.height()
                + self.dimensions.max_y()
                - 1.);
        let x = self.dimensions.width() * chunk_x + coord.x();
        let y = (self.dimensions.height() * self.dimensions.max_y())
            - (self.dimensions.height() * chunk_y)
            + coord.y();
        Some(Vec3::new(x, y, coord.z()))
    }
}

#[derive(Bundle, Debug)]
pub struct TileMapComponents {
    pub tile_map: TileMap,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// The event handling system for the `TileMap` which takes the types `Tile`, `Chunk`, and `TileMap`.
pub fn map_system(
    mut commands: Commands,
    mut chunks: ResMut<Assets<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Entity, &mut TileMap)>,
) {
    for (map_entity, mut map) in query.iter_mut() {
        map.events_update();
        let mut new_chunks = HashSet::<(usize, Handle<Chunk>)>::default();
        let mut refresh_chunks = HashSet::<Handle<Chunk>>::default();
        let mut modified_chunks = Vec::new();
        let mut despawned_chunks = HashSet::<(Handle<Chunk>, Entity)>::default();
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

        let mut chunk_entities = Vec::with_capacity(new_chunks.len());
        for (idx, chunk_handle) in new_chunks.iter() {
            let chunk = chunks.get(chunk_handle).unwrap();
            let mesh = meshes.get_mut(chunk.mesh()).unwrap();
            let map_coord = map.dimensions().decode_coord_unchecked(*idx);
            let map_center = map.dimensions().center();

            let (tile_indexes, tile_colors) = chunk.tiles_to_renderer_parts();
            mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, tile_indexes.into());
            mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, tile_colors.into());
            let translation = Vec3::new(
                (map_coord.x() - map_center.x() + 0.5)
                    * map.tile_dimensions().width()
                    * map.chunk_dimensions().width(),
                (map_coord.y() - map_center.y() + 0.5)
                    * map.tile_dimensions().height()
                    * map.chunk_dimensions().height(),
                1.,
            );
            let chunk_entity = commands
                .spawn(ChunkComponents {
                    texture_atlas: map.texture_atlas_handle().clone_weak(),
                    draw: Default::default(),
                    render_pipelines: Default::default(),
                    main_pass: Default::default(),
                    mesh: chunk.mesh().clone_weak(),
                    transform: Transform::from_translation(translation),
                    global_transform: Default::default(),
                })
                .current_entity()
                .expect("Chunk entity unexpected does not exist.");
            chunk_entities.push(chunk_entity);
        }
        commands.push_children(*map_entity, &chunk_entities);

        for (chunk_handle, setter) in modified_chunks.iter() {
            let chunk = chunks.get_mut(chunk_handle).unwrap();
            for (setter_coord, setter_tile) in setter.iter() {
                let idx = setter_coord.to_index(
                    map.chunk_dimensions().width(),
                    map.chunk_dimensions().height(),
                );
                chunk.set_tile(idx, *setter_tile);
            }

            let mesh = meshes.get_mut(chunk.mesh()).unwrap();
            let (tile_indexes, tile_colors) = chunk.tiles_to_renderer_parts();
            mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, tile_indexes.into());
            mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, tile_colors.into());
        }

        for (_chunk_handle, entity) in despawned_chunks.iter() {
            commands.despawn(*entity);
        }

        for (index, entity) in removed_chunks.iter() {
            map.remove_chunk_handle(*index);
            commands.despawn(*entity);
        }
    }
}
