use crate::{
    chunk::{Chunk, LayerKind},
    coord::{ToCoord2, ToCoord3, ToIndex},
    dimensions::{DimensionError, Dimensions2, Dimensions3},
    entity::ChunkComponents,
    lib::*,
    mesh::ChunkMesh,
    tile::Tile,
    tile_setter::TileSetter,
};

#[derive(Clone, PartialEq)]
/// The kinds of errors that can occur for a `[MapError]`.
pub enum ErrorKind {
    /// If the coordinate or index is out of bounds.
    DimensionError(DimensionError),
    /// If a layer already exists this error is returned.
    LayerExists(usize),
    /// If a layer does not already exist this error is returned.
    LayerDoesNotExist(usize),
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use ErrorKind::*;
        match self {
            DimensionError(err) => err.fmt(f),
            LayerExists(n) => write!(
                f,
                "layer {} already exists, try `remove_layer` or `move_layer` first",
                n
            ),
            LayerDoesNotExist(n) => write!(f, "layer {} does not exist, try `add_layer` first", n),
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
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }
}

impl From<DimensionError> for MapError {
    fn from(err: DimensionError) -> MapError {
        MapError::new(ErrorKind::DimensionError(err))
    }
}

/// A map result.
pub type MapResult<T> = Result<T, MapError>;

/// Events that happen on a `Chunk` by index value.
#[derive(Debug)]
pub(crate) enum MapEvent {
    /// To be used when a chunk is created.
    CreatedChunk {
        /// The index of the chunk.
        index: usize,
        /// The Handle of the Chunk.
        handle: Handle<Chunk>,
    },
    /// An event when a layer is created for all chunks.
    AddedLayer {
        z: usize,
        kind: LayerKind,
    },
    MovedLayer {
        from_z: usize,
        to_z: usize,
    },
    /// An event when a layer is removed for all chunks.
    RemovedLayer {
        z: usize,
    },
    /// An event when a chunk had been modified by changing tiles.
    ModifiedChunk {
        /// The map index where the chunk needs to be stored.
        handle: Handle<Chunk>,
        /// The `TileSetter` that is used to set all the tiles.
        setter: TileSetter,
    },
    /// An event when a chunk is spawned.
    SpawnedChunk {
        handle: Handle<Chunk>,
    },
    /// If the chunk needs to be despawned, this event is used.
    DespawnedChunk {
        /// The `Handle` of the `Chunk`.
        handle: Handle<Chunk>,
        // /// The `Entity` that needs to be despawned.
        // entity: Entity,
    },
    /// If the chunk needs to be removed.
    ///
    /// # Warning
    /// This is destructive action! All data will be dropped and removed.
    RemovedChunk {
        /// The map index where the chunk needs to be removed.
        handle: Handle<Chunk>,
        // /// The `Entity` that needs to be despawned.
        // entity: Entity,
    },
}

/// A TileMap which maintains chunks and its tiles within.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct TileMap {
    dimensions: Vec2,
    chunk_dimensions: Vec3,
    tile_dimensions: Vec2,
    current_depth: usize,
    layers: Vec<Option<LayerKind>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    chunks: HashMap<usize, Handle<Chunk>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    entities: HashMap<usize, Vec<Entity>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    events: Events<MapEvent>,
    #[cfg_attr(feature = "serde", serde(skip))]
    texture_atlas: Handle<TextureAtlas>,
    // NOTE: If there is a better way to keep track of spawned chunks, that
    // be swell. Perhaps spawning the chunks themselves with the layers as
    // children?
    #[cfg_attr(feature = "serde", serde(skip))]
    spawned_chunks: Vec<usize>,
}

impl TypeUuid for TileMap {
    const TYPE_UUID: Uuid = Uuid::from_u128(109481186966523254410691740507722642628);
}

impl TileMap {
    /// Returns a new WorldMap with the types `Tile` and `Chunk`.
    ///
    /// It takes in dimensions for itself, the chunk, and the tile. These must
    /// be uniform in order for the TileMap not to render with gaps in places
    /// that it should not. The `TextureAtlas` handle is used to get the correct
    /// sprite sheet to be used.
    ///
    /// # Example
    /// ```
    /// use bevy_tilemap::prelude::*;
    /// use bevy::prelude::*;
    /// use bevy::type_registry::TypeUuid;
    ///
    /// // Tile's dimensions in pixels
    /// let tile_dimensions = Vec2::new(32., 32.);
    /// // Chunk's dimensions in tiles
    /// let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// // Tile map's dimensions in chunks
    /// let tile_map_dimensions = Vec2::new(1., 1.,);
    /// // Handle from the sprite sheet you want
    /// let atlas_handle = Handle::weak_from_u64(TextureAtlas::TYPE_UUID, 1234567890);
    ///
    /// let tile_map = TileMap::new(
    ///     tile_map_dimensions,
    ///     chunk_dimensions,
    ///     tile_dimensions,
    ///     atlas_handle,
    /// );
    /// ```
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
            layers: vec![None; 20],
            current_depth: 0,
            chunks: HashMap::with_capacity(capacity),
            entities: HashMap::default(),
            events: Events::<MapEvent>::default(),
            texture_atlas,
            spawned_chunks: Vec::new(),
        }
    }

    // NOTE: Keep this in, it is a future wanted method.
    // /// Sets the dimensions of the `TileMap`.
    // ///
    // /// These dimensions must be in chunks. This is useful if for whatever
    // /// reason you need to resize the TileMap prior to settling down on it.
    // ///
    // /// # Warning
    // /// This is not intended to be done after a TileMap has been initialized.
    // /// Will not work as expected as it is missing methods to change all the
    // /// chunk translations as well.
    // ///
    // /// # Examples
    // /// ```
    // ///
    // /// ```
    // pub fn set_dimensions(&mut self, dimensions: Vec2) {
    //     self.handles = vec![None; (dimensions.width() * dimensions.height()) as usize];
    //     self.dimensions = dimensions;
    // }

    /// Sets the sprite sheet, or `TextureAtlas` for use in the `TileMap`.
    ///
    /// This can be used if the need to swap the sprite sheet for another is
    /// wanted.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::prelude::*;
    /// # use bevy::asset::Handle;
    /// # use bevy::math::{Vec2, Vec3};
    /// # use bevy::sprite::TextureAtlas;
    /// # use bevy::type_registry::TypeUuid;
    /// #
    /// # // Tile's dimensions in pixels
    /// # let tile_dimensions = Vec2::new(32., 32.);
    /// # // Chunk's dimensions in tiles
    /// # let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// # // Tile map's dimensions in chunks
    /// # let tile_map_dimensions = Vec2::new(1., 1.,);
    /// # // Handle from the sprite sheet you want
    /// # let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    /// #
    /// # let mut tile_map = TileMap::new(
    /// #     tile_map_dimensions,
    /// #     chunk_dimensions,
    /// #     tile_dimensions,
    /// #     atlas_handle,
    /// # );
    /// #
    /// let new_atlas_handle = Handle::weak_from_u64(TextureAtlas::TYPE_UUID, 0987654321);
    ///
    /// tile_map.set_texture_atlas(new_atlas_handle);
    /// ```
    pub fn set_texture_atlas(&mut self, handle: Handle<TextureAtlas>) {
        self.texture_atlas = handle;
    }

    /// Returns a reference the `Handle` of the `TextureAtlas`.
    ///
    /// The Handle is used to get the correct sprite sheet that is used for this
    /// `Tile Map` with the renderer.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::type_registry::TypeUuid;
    /// #
    /// # // Tile's dimensions in pixels
    /// # let tile_dimensions = Vec2::new(32., 32.);
    /// # // Chunk's dimensions in tiles
    /// # let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// # // Tile map's dimensions in chunks
    /// # let tile_map_dimensions = Vec2::new(1., 1.,);
    /// # // Handle from the sprite sheet you want
    /// # let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    /// #
    /// # let mut tile_map = TileMap::new(
    /// #     tile_map_dimensions,
    /// #     chunk_dimensions,
    /// #     tile_dimensions,
    /// #     atlas_handle,
    /// # );
    /// #
    /// let texture_atlas: &Handle<TextureAtlas> = tile_map.texture_atlas();
    /// ```
    pub fn texture_atlas(&self) -> &Handle<TextureAtlas> {
        &self.texture_atlas
    }

    /// Constructs a new `Chunk` and stores it at a coordinate position.
    ///
    /// It requires that you give it either an index or a Vec2 or Vec3
    /// coordinate. It then automatically sets both a sized mesh and chunk for
    /// use based on the parameters set in the parent `TileMap`.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::type_registry::TypeUuid;
    /// #
    /// # // Tile's dimensions in pixels
    /// # let tile_dimensions = Vec2::new(32., 32.);
    /// # // Chunk's dimensions in tiles
    /// # let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// # // Tile map's dimensions in chunks
    /// # let tile_map_dimensions = Vec2::new(1., 1.,);
    /// # // Handle from the sprite sheet you want
    /// # let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    /// #
    /// # let mut tile_map = TileMap::new(
    /// #    tile_map_dimensions,
    /// #    chunk_dimensions,
    /// #    tile_dimensions,
    /// #    atlas_handle,
    /// # );
    ///
    /// // Add some chunks.
    /// tile_map.new_chunk(0);
    /// tile_map.new_chunk(1);
    /// tile_map.new_chunk(2);
    /// ```
    pub fn new_chunk<I: ToIndex>(&mut self, v: I) -> MapResult<()> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
        self.dimensions.check_index(index)?;

        let handle: Handle<Chunk> = Handle::weak(HandleId::random::<Chunk>());
        self.chunks.insert(index, handle.clone_weak());

        self.events.send(MapEvent::CreatedChunk { index, handle });

        Ok(())
    }

    /// Adds a layer to the `TileMap` and inner chunks.
    ///
    /// This method takes in a `LayerKind`, coordinate as either an index or
    /// Vec2, as well as a specified Z layer that the layer needs to be set to.
    /// If the layer is already the specified layer's kind, then nothing
    /// happens.
    ///
    /// # Errors
    ///
    /// If a layer is set and a different layer already exists at that Z layer
    /// then an error is returned regarding that.
    ///
    /// # Panics
    ///
    /// If the tile length is not the same length as the expected tile length
    /// per chunk, then a panic is thrown. This is important as it will lead to
    /// future panics if it is not caught here.
    pub fn add_layer(&mut self, kind: LayerKind, z_layer: usize) -> MapResult<()> {
        // assert_eq!(
        //     tiles.len(),
        //     self.chunk_tile_len(),
        //     "The tiles length must be equal to the expected tile length per chunk."
        // );

        if let Some(some_kind) = self.layers[z_layer] {
            return if some_kind == kind {
                Ok(())
            } else {
                Err(ErrorKind::LayerExists(z_layer).into())
            };
        }

        self.layers[z_layer] = Some(kind);

        self.events.send(MapEvent::AddedLayer { z: z_layer, kind });

        Ok(())
    }

    /// Moves a layer from one Z level to another.
    ///
    /// # Errors
    ///
    /// If the destination exists, it will throw an error. Likewise, if the
    /// origin does not exist, it also will throw an error.
    pub fn move_layer(&mut self, from_z: usize, to_z: usize) -> MapResult<()> {
        if self.layers[to_z].is_some() {
            return Err(ErrorKind::LayerExists(to_z).into());
        };

        if self.layers[from_z].is_none() {
            return Err(ErrorKind::LayerDoesNotExist(from_z).into());
        };

        self.layers[to_z] = self.layers[from_z];
        self.layers[from_z] = None;

        self.events.send(MapEvent::MovedLayer { from_z, to_z });

        Ok(())
    }

    /// Removes a layer from the `TileMap` and inner chunks.
    ///
    /// **Warning**: This is destructive if you have tiles that exist on that
    /// layer. If you want to add them back in, better to use the `move_layer`
    /// method instead.
    ///
    /// This method takes in a Z layer which is then flagged for deletion. If
    /// the layer already does not exist, it does nothing.
    pub fn remove_layer(&mut self, z: usize) {
        if self.layers[z].is_none() {
            return;
        }

        self.layers[z] = None;

        self.events.send(MapEvent::RemovedLayer { z })
    }

    /// Spawns a stored chunk at a given index or coordinate.
    ///
    /// This **only** spawns stored chunks. If it is required to construct a new
    /// chunk then use `new_chunk` first.
    ///
    /// # Errors
    ///
    /// If the coordinate or index is out of bounds, an error will be returned.
    pub fn spawn_chunk<I: ToIndex>(&mut self, v: I) -> MapResult<()> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
        self.dimensions.check_index(index)?;

        let handle = self.chunks.get(&index).unwrap();

        self.events.send(MapEvent::SpawnedChunk {
            handle: handle.clone_weak(),
        });

        Ok(())
    }

    /// De-spawns a spawned chunk at a given index or coordinate.
    pub fn despawn_chunk<I: ToIndex>(&mut self, v: usize) -> MapResult<()> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
        self.dimensions.check_index(index)?;

        let handle = self.chunks.get(&index).unwrap();

        self.events.send(MapEvent::DespawnedChunk {
            handle: handle.clone_weak(),
        });

        Ok(())
    }

    #[doc(hidden)]
    #[deprecated(since = "0.2.0", note = "please use `new_chunk` instead")]
    pub fn new_chunk_with_tiles<I: ToIndex>(
        &mut self,
        v: I,
        _tiles: Vec<Tile>,
    ) -> MapResult<()> {
        self.new_chunk(v)
    }

    /// Destructively removes a `Chunk` at a coordinate position.
    ///
    /// Internally, this sends an event to the `TileMap`'s system flagging which
    /// chunks must be removed by index and entity. A chunk is not recoverable
    /// if this action is done.
    ///
    /// # Examples
    /// ```no_run
    /// # use bevy_tilemap::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::type_registry::TypeUuid;
    /// #
    /// # // Tile's dimensions in pixels
    /// # let tile_dimensions = Vec2::new(32., 32.);
    /// # // Chunk's dimensions in tiles
    /// # let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// # // Tile map's dimensions in chunks
    /// # let tile_map_dimensions = Vec2::new(1., 1.,);
    /// # // Handle from the sprite sheet you want
    /// # let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    /// #
    /// # let mut tile_map = TileMap::new(
    /// #    tile_map_dimensions,
    /// #    chunk_dimensions,
    /// #    tile_dimensions,
    /// #    atlas_handle,
    /// # );
    /// // Add some chunks.
    /// tile_map.new_chunk(0);
    /// tile_map.new_chunk(1);
    /// tile_map.new_chunk(2);
    ///
    /// // Remove the same chunks in the same frame. Do note that adding then
    /// // removing in the same frame will prevent the entity from spawning at
    /// // all.
    /// tile_map.remove_chunk(0);
    /// tile_map.remove_chunk(1);
    /// tile_map.remove_chunk(2);
    /// ```
    pub fn remove_chunk<I: ToIndex>(&mut self, v: I) -> MapResult<()> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.y());
        self.dimensions.check_index(index)?;

        let handle = self.chunks.get(&index).unwrap();

        self.events.send(MapEvent::RemovedChunk {
            handle: handle.clone_weak(),
        });

        Ok(())
    }

    /// Sets a single tile at a coordinate position and checks if it the request is inbounds.
    ///
    /// For convenience, this does not require to use a TileSetter which is beneficial for multiple
    /// tiles. If that is preferred, do use [set_tiles] instead.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::type_registry::TypeUuid;
    /// #
    /// # // Tile's dimensions in pixels
    /// # let tile_dimensions = Vec2::new(32., 32.);
    /// # // Chunk's dimensions in tiles
    /// # let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// # // Tile map's dimensions in chunks
    /// # let tile_map_dimensions = Vec2::new(1., 1.,);
    /// # // Handle from the sprite sheet you want
    /// # let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    /// #
    /// # let mut tile_map = TileMap::new(
    /// #    tile_map_dimensions,
    /// #    chunk_dimensions,
    /// #    tile_dimensions,
    /// #    atlas_handle,
    /// # );
    /// // Add a chunk
    /// tile_map.new_chunk(0);
    ///
    /// // Set a single tile and unwrap the result
    /// tile_map.set_tile(Vec3::new(15., 15., 0.), Tile::new(1), 0).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the given coordinate or index is out of bounds.
    pub fn set_tile<I: ToIndex + ToCoord3>(&mut self, v: I, tile: Tile, z: usize) -> MapResult<()> {
        let coord = v.to_coord3(self.dimensions.width(), self.dimensions.height());
        self.set_tiles(TileSetter::from(vec![(coord, tile, z)]))
    }

    /// Sets many tiles using a `TileSetter`.
    ///
    /// If setting a single tile is more preferable, then use the [set_tile]
    /// method instead.
    ///
    /// # Errors
    ///
    /// Returns an error if the given coordinate or index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_tilemap::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::type_registry::TypeUuid;
    /// #
    /// # // Tile's dimensions in pixels
    /// # let tile_dimensions = Vec2::new(32., 32.);
    /// # // Chunk's dimensions in tiles
    /// # let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// # // Tile map's dimensions in chunks
    /// # let tile_map_dimensions = Vec2::new(1., 1.,);
    /// # // Handle from the sprite sheet you want
    /// # let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    /// #
    /// # let mut tile_map = TileMap::new(
    /// #    tile_map_dimensions,
    /// #    chunk_dimensions,
    /// #    tile_dimensions,
    /// #    atlas_handle,
    /// # );
    /// // Add a chunk
    /// tile_map.new_chunk(0);
    ///
    /// let mut new_tiles = TileSetter::new();
    /// new_tiles.push(Vec3::new(1., 1., 0.), Tile::new(1), 0);
    /// new_tiles.push(Vec3::new(2., 2., 0.), Tile::new(2), 0);
    /// new_tiles.push(Vec3::new(3., 3., 0.), Tile::new(3), 0);
    ///
    /// // Set multiple tiles and unwrap the result
    /// tile_map.set_tiles(new_tiles).unwrap();
    /// ```
    pub fn set_tiles(&mut self, setter: TileSetter) -> MapResult<()> {
        let mut tiles_map: HashMap<usize, TileSetter> = HashMap::default();
        for (setter_coord, setter_tile, z) in setter.iter() {
            let chunk_coord = self.tile_coord_to_chunk_coord(*setter_coord);
            let chunk_index =
                chunk_coord.to_index(self.dimensions.width(), self.dimensions.height());
            self.dimensions.check_index(chunk_index)?;

            let tile_y = setter_coord.y() / self.chunk_dimensions.height();
            let map_coord = Vec2::new(
                (setter_coord.x() / self.chunk_dimensions.width()).floor(),
                self.dimensions.max_y() - (self.dimensions.max_y() as f32 - tile_y),
            );
            let x = setter_coord.x() - (map_coord.x() * self.chunk_dimensions.width());
            let y = setter_coord.y() - chunk_coord.y() * self.chunk_dimensions.height();
            let coord = Vec3::new(x, y, setter_coord.z());
            self.chunk_dimensions.check_coord(&coord)?;

            if let Some(setters) = tiles_map.get_mut(&chunk_index) {
                setters.push(coord, *setter_tile, *z);
            } else {
                let mut setter = TileSetter::new();
                setter.push(coord, *setter_tile, *z);
                tiles_map.insert(chunk_index, setter);
            }
        }

        for (index, setter) in tiles_map {
            let handle = self.chunks.get(&index).unwrap();
            self.events.send(MapEvent::ModifiedChunk {
                handle: handle.clone_weak(),
                setter,
            })
        }
        Ok(())
    }

    /// Returns the center tile of the `Map` as a `Vec2` `Tile` coordinate.
    ///
    /// This returns the center of the map rounded down.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_tilemap::prelude::*;
    /// # use bevy::prelude::*;
    /// # use bevy::type_registry::TypeUuid;
    /// #
    /// # // Tile's dimensions in pixels
    /// # let tile_dimensions = Vec2::new(32., 32.);
    /// # // Chunk's dimensions in tiles
    /// # let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// # // Tile map's dimensions in chunks
    /// # let tile_map_dimensions = Vec2::new(1., 1.,);
    /// # // Handle from the sprite sheet you want
    /// # let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    /// #
    /// # let mut tile_map = TileMap::new(
    /// #    tile_map_dimensions,
    /// #    chunk_dimensions,
    /// #    tile_dimensions,
    /// #    atlas_handle,
    /// # );
    ///
    /// let center: Vec2 = tile_map.center_tile_coord();
    ///
    /// assert_eq!(Vec2::new(16., 16.), center);
    /// ```
    pub fn center_tile_coord(&self) -> Vec2 {
        let x = self.dimensions.width() / 2. * self.chunk_dimensions.width();
        let y = self.dimensions.height() / 2. * self.chunk_dimensions.height();
        Vec2::new(x.floor(), y.floor())
    }

    /// Takes a tile coordinate and changes it into a chunk coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_tilemap::prelude::*;
    /// use bevy::prelude::*;
    /// use bevy::type_registry::TypeUuid;
    ///
    /// // Tile's dimensions in pixels
    /// let tile_dimensions = Vec2::new(32., 32.);
    /// // Chunk's dimensions in tiles
    /// let chunk_dimensions = Vec3::new(32., 32., 0.);
    /// // Tile map's dimensions in chunks
    /// let tile_map_dimensions = Vec2::new(3., 3.,);
    /// // Handle from the sprite sheet you want
    /// let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    ///
    /// let mut tile_map = TileMap::new(
    ///    tile_map_dimensions,
    ///    chunk_dimensions,
    ///    tile_dimensions,
    ///    atlas_handle,
    /// );
    ///
    /// let tile_coord = Vec3::new(15., 15., 0.);
    ///
    /// let chunk_coord = tile_map.tile_coord_to_chunk_coord(tile_coord);
    ///
    /// assert_eq!(Vec2::new(0., 0.), chunk_coord);
    /// ```
    pub fn tile_coord_to_chunk_coord(&self, coord: Vec3) -> Vec2 {
        let x = (coord.x() / self.chunk_dimensions.y()).floor();
        let y = (coord.y() / self.chunk_dimensions.x()).floor();
        Vec2::new(x, y)
    }

    // FIXME: These need to be changed as they will be inaccurate if the
    // Transform of the TileMap is changed from 0,0.
    // /// Takes a translation and calculates the `Tile` coordinate.
    // ///
    // /// # Examples
    // /// ```
    // /// # use bevy_tilemap::TileMap;
    // /// # use bevy::prelude::*;
    // /// # use bevy::type_registry::TypeUuid;
    // /// #
    // /// # // Tile's dimensions in pixels
    // /// # let tile_dimensions = Vec2::new(32., 32.);
    // /// # // Chunk's dimensions in tiles
    // /// # let chunk_dimensions = Vec3::new(32., 32., 0.);
    // /// # // Tile map's dimensions in chunks
    // /// # let tile_map_dimensions = Vec2::new(1., 1.,);
    // /// # // Handle from the sprite sheet you want
    // /// # let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    // /// #
    // /// # let mut tile_map = TileMap::new(
    // /// #    tile_map_dimensions,
    // /// #    chunk_dimensions,
    // /// #    tile_dimensions,
    // /// #    atlas_handle,
    // /// # );
    // ///
    // /// let translation = Vec3::new(0., 0., 0.);
    // /// let tile_coord = tile_map.translation_to_tile_coord(translation);
    // ///
    // /// assert_eq!(Vec2::new(16., 16.), tile_coord);
    // /// ```
    // pub fn translation_to_tile_coord(&self, translation: Vec3) -> Vec2 {
    //     let center = self.center_tile_coord();
    //     let x = translation.x() / self.tile_dimensions.width() as f32 + center.x();
    //     let y = translation.y() / self.tile_dimensions.height() as f32 + center.y();
    //     Vec2::new(x, y)
    // }

    // FIXME: These need to be changed as they will be inaccurate if the
    // Transform of the TileMap is changed from 0,0.
    // /// Takes a translation and calculates the `Chunk` coordinate.
    // ///
    // /// # Examples
    // /// ```
    // /// use bevy_tilemap::TileMap;
    // /// use bevy::prelude::*;
    // /// use bevy::type_registry::TypeUuid;
    // ///
    // /// // Tile's dimensions in pixels
    // /// let tile_dimensions = Vec2::new(32., 32.);
    // /// // Chunk's dimensions in tiles
    // /// let chunk_dimensions = Vec3::new(32., 32., 0.);
    // /// // Tile map's dimensions in chunks
    // /// let tile_map_dimensions = Vec2::new(3., 3.,);
    // /// // Handle from the sprite sheet you want
    // /// let atlas_handle = Handle::weak_from_u64(TileMap::TYPE_UUID, 1234567890);
    // ///
    // /// let mut tile_map = TileMap::new(
    // ///    tile_map_dimensions,
    // ///    chunk_dimensions,
    // ///    tile_dimensions,
    // ///    atlas_handle,
    // /// );
    // ///
    // /// let translation = Vec3::new(0., 0., 0.);
    // /// let chunk_coord = tile_map.translation_to_chunk_coord(translation);
    // ///
    // /// assert_eq!(Vec2::new(1., 1.), chunk_coord);
    // /// ```
    // pub fn translation_to_chunk_coord(&self, translation: Vec3) -> Vec2 {
    //     let center = self.dimensions.center();
    //     let x = translation.x() as i32
    //         / (self.tile_dimensions.width() as i32 * self.chunk_dimensions.width() as i32)
    //         + center.x() as i32;
    //     let y = translation.y() as i32
    //         / (self.tile_dimensions.height() as i32 * self.chunk_dimensions.height() as i32)
    //         + center.y() as i32;
    //     Vec2::new(x as f32, y as f32)
    // }

    /// Returns the dimensions of the `TileMap`.
    pub fn dimensions(&self) -> Vec2 {
        self.dimensions
    }

    /// Returns the chunk dimensions of the `TileMap`.
    pub fn chunk_dimensions(&self) -> Vec3 {
        self.chunk_dimensions
    }

    /// A Chunk's size in pixels.
    pub fn chunk_pixel_size(&self) -> Vec2 {
        Vec2::new(
            self.tile_dimensions.width() * self.chunk_dimensions.width(),
            self.tile_dimensions.height() * self.chunk_dimensions.height(),
        )
    }

    /// Returns the tile dimensions of the `TileMap`.
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

/// The event handling system for the `TileMap` which takes the types `Tile`, `Chunk`, and `TileMap`.
pub fn map_system(
    mut commands: Commands,
    mut chunks: ResMut<Assets<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut TileMap)>,
) {
    for (map_entity, mut map) in query.iter_mut() {
        map.events.update();

        let mut new_chunks = Vec::new();
        let mut added_layers = Vec::new();
        let mut moved_layers = Vec::new();
        let mut removed_layers = Vec::new();
        let mut modified_chunks = Vec::new();
        let mut spawned_chunks = Vec::new();
        let mut despawned_chunks = Vec::new();
        let mut removed_chunks = Vec::new();
        let mut reader = map.events.get_reader();
        for event in reader.iter(&map.events) {
            use MapEvent::*;
            match event {
                CreatedChunk {
                    ref index,
                    ref handle,
                } => {
                    new_chunks.push((*index, handle.clone_weak()));
                }
                AddedLayer { ref z, ref kind } => {
                    added_layers.push((*z, *kind));
                }
                MovedLayer {
                    ref from_z,
                    ref to_z,
                } => {
                    moved_layers.push((*from_z, *to_z));
                }
                RemovedLayer { ref z } => {
                    removed_layers.push(*z);
                }
                ModifiedChunk {
                    ref handle,
                    ref setter,
                } => {
                    modified_chunks.push((handle.clone_weak(), setter.clone()));
                }
                SpawnedChunk { ref handle } => {
                    spawned_chunks.push(handle.clone_weak());
                }
                DespawnedChunk { ref handle } => {
                    despawned_chunks.push(handle.clone_weak());
                }
                RemovedChunk { ref handle } => {
                    removed_chunks.push(handle.clone_weak());
                }
            }
        }

        for (index, handle) in new_chunks.iter() {
            let coord = index.to_coord2(map.dimensions.width(), map.dimensions.height());
            let mut chunk = Chunk::new(coord, map.chunk_dimensions, map.layers.len());
            chunk.add_layer(LayerKind::Dense, 0);
            let handle = chunks.set(handle.clone_weak(), chunk);
            map.chunks.insert(*index, handle);
        }

        for (z, kind) in added_layers.iter() {
            for handle in map.chunks.values() {
                let chunk = chunks.get_mut(handle).unwrap();
                chunk.add_layer(*kind, *z);
            }
        }

        for (from_z, to_z) in moved_layers.iter() {
            for handle in map.chunks.values() {
                let chunk = chunks.get_mut(handle).unwrap();
                chunk.move_layer(*from_z, *to_z);
            }
        }

        for z in removed_layers.iter() {
            for handle in map.chunks.values() {
                let chunk = chunks.get_mut(handle).unwrap();
                chunk.remove_layer(*z);
            }
        }

        for (handle, setter) in modified_chunks.iter() {
            let chunk = chunks.get_mut(handle).unwrap();
            for (coord, tile, z_layer) in setter.iter() {
                let index =
                    coord.to_index(map.chunk_dimensions.width(), map.chunk_dimensions.height());
                chunk.set_tile(*z_layer, index, *tile);
            }
        }

        for handle in spawned_chunks.iter() {
            let chunk = chunks.get_mut(handle).unwrap();
            let mut entities = Vec::with_capacity(new_chunks.len());
            for z in 0..map.layers.len() - 1 {
                let mut mesh = Mesh::from(ChunkMesh::new(map.chunk_dimensions));
                let (indexes, colors) = if let Some(parts) = chunk.tiles_to_renderer_parts(z) {
                    parts
                } else {
                    continue;
                };
                mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes.into());
                mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors.into());
                let mesh_handle = meshes.add(mesh);
                chunk.set_mesh(z, mesh_handle.clone());

                let idx = chunk
                    .coord()
                    .to_index(map.dimensions.width(), map.dimensions.height());

                let map_coord = map.dimensions().decode_coord_unchecked(idx);
                let map_center = map.dimensions().center();

                let translation = Vec3::new(
                    (map_coord.x() - map_center.x() + 0.5)
                        * map.tile_dimensions().width()
                        * map.chunk_dimensions().width(),
                    (map_coord.y() - map_center.y() + 0.5)
                        * map.tile_dimensions().height()
                        * map.chunk_dimensions().height(),
                    z as f32,
                );
                let entity = commands
                    .spawn(ChunkComponents {
                        texture_atlas: map.texture_atlas().clone_weak(),
                        chunk_dimensions: map.chunk_dimensions().into(),
                        mesh: mesh_handle.clone_weak(),
                        transform: Transform::from_translation(translation),
                        ..Default::default()
                    })
                    .current_entity()
                    .expect("Chunk entity unexpected does not exist.");

                map.spawned_chunks.push(idx);
                chunk.add_entity(z, entity);
                entities.push(entity);
            }
            commands.push_children(map_entity, &entities);
        }

        for handle in despawned_chunks.iter() {
            let chunk = chunks.get_mut(handle).unwrap();
            let index = chunk
                .coord()
                .to_index(map.dimensions.width(), map.dimensions.height());
            for entity in chunk.get_entities() {
                commands.despawn(entity);
            }

            let spawned_index = map.spawned_chunks.iter().position(|x| *x == index).unwrap();
            map.spawned_chunks.remove(spawned_index);
        }

        for handle in removed_chunks.iter() {
            let chunk = chunks.get_mut(handle).unwrap();
            for entity in chunk.get_entities() {
                commands.despawn(entity);
            }
            let index = chunk
                .coord()
                .to_index(map.dimensions.width(), map.dimensions.height());
            map.chunks.remove(&index);

            let spawned_index = map.spawned_chunks.iter().position(|x| *x == index).unwrap();
            map.spawned_chunks.remove(spawned_index);
        }
    }
}
