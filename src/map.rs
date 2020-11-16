use crate::{
    chunk::Chunk,
    coord::{ToCoord3, ToIndex},
    dimensions::{DimensionError, DimensionResult, Dimensions2, Dimensions3},
    entity::ChunkComponents,
    lib::*,
    mesh::ChunkMesh,
    tile::{Tile, TileSetter},
};

#[derive(Clone, PartialEq)]
/// The kinds of errors that can occur for a `[MapError]`.
pub enum ErrorKind {
    /// If the coordinate or index is out of bounds.
    DimensionError(DimensionError),
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use ErrorKind::*;
        match self {
            DimensionError(err) => err.fmt(f),
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
pub enum MapEvent {
    /// To be used when a chunk is created.
    Created {
        /// The map index where the chunk needs to be stored.
        index: usize,
        // /// The Handle of the Chunk.
        // handle: Handle<Chunk>,
        /// The vector of `Tile`s for the `Chunk`.
        tiles: Vec<Tile>,
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
        /// The map index where the chunk needs to be stored.
        index: usize,
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

/// A TileMap which maintains chunks and its tiles within.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, RenderResources)]
pub struct TileMap {
    #[render_resources(ignore)]
    dimensions: Vec2,
    chunk_dimensions: Vec3,
    #[render_resources(ignore)]
    tile_dimensions: Vec2,
    #[cfg_attr(feature = "serde", serde(skip))]
    // Should change to HashSet when merged into bevy
    #[render_resources(ignore)]
    chunks: Vec<Option<Handle<Chunk>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    #[render_resources(ignore)]
    entities: HashMap<usize, Entity>,
    #[cfg_attr(feature = "serde", serde(skip))]
    #[render_resources(ignore)]
    events: Events<MapEvent>,
    #[cfg_attr(feature = "serde", serde(skip))]
    #[render_resources(ignore)]
    texture_atlas: Handle<TextureAtlas>,
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
    /// use bevy_tilemap::TileMap;
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
            chunks: vec![None; capacity],
            entities: HashMap::default(),
            events: Events::<MapEvent>::default(),
            texture_atlas,
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
    /// # use bevy_tilemap::TileMap;
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
    /// # use bevy_tilemap::TileMap;
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
    /// # use bevy_tilemap::TileMap;
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
    pub fn new_chunk<I: ToIndex>(&mut self, v: I) -> DimensionResult<()> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
        self.dimensions.check_index(index)?;

        let tiles = vec![
            Tile::new(0);
            (self.chunk_dimensions().width() * self.chunk_dimensions().height())
                as usize
        ];
        self.events.send(MapEvent::Created { index, tiles });

        Ok(())
    }

    /// Constructs a new `Chunk` and stores it at a coordinate position with
    /// tiles.
    ///
    /// It requires that you give it either an index or a Vec2 or Vec3
    /// coordinate as well as a vector of `Tile`s. It then automatically sets
    /// both a sized mesh and chunk for use based on the parameters set in the
    /// parent `TileMap`.
    ///
    /// # Panics
    ///
    /// This method will panic if you attempt to add a chunk to an out of bounds
    /// index location or coordinate.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::TileMap;
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
    /// use bevy_tilemap::Tile;
    ///
    /// let tiles = vec![Tile::new(0); 32];
    ///
    /// // Add some chunks.
    /// tile_map.new_chunk_with_tiles(0, tiles.clone());
    /// tile_map.new_chunk_with_tiles(1, tiles.clone());
    /// tile_map.new_chunk_with_tiles(2, tiles);
    /// ```
    pub fn new_chunk_with_tiles<I: ToIndex>(
        &mut self,
        v: I,
        tiles: Vec<Tile>,
    ) -> DimensionResult<()> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.height());
        self.dimensions.check_index(index)?;

        self.events.send(MapEvent::Created { index, tiles });

        Ok(())
    }

    /// Destructively removes a `Chunk` at a coordinate position.
    ///
    /// Internally, this sends an event to the `TileMap`'s system flagging which
    /// chunks must be removed by index and entity. A chunk is not recoverable
    /// if this action is done.
    ///
    /// # Examples
    /// ```no_run
    /// # use bevy_tilemap::TileMap;
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
    /// use bevy_tilemap::Tile;
    ///
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
    pub fn remove_chunk<I: ToIndex>(&mut self, v: I) -> DimensionResult<()> {
        let index = v.to_index(self.dimensions.width(), self.dimensions.y());
        self.dimensions.check_index(index)?;

        let entity = *self.entities.get(&index).unwrap();
        self.events.send(MapEvent::Removed { index, entity });

        Ok(())
    }

    /// Sets a single tile at a coordinate position and checks if it the request is inbounds.
    ///
    /// For convenience, this does not require to use a TileSetter which is beneficial for multiple
    /// tiles. If that is preferred, do use [set_tiles] instead.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::TileMap;
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
    /// use bevy_tilemap::Tile;
    ///
    /// // Add a chunk
    /// tile_map.new_chunk(0);
    ///
    /// // Set a single tile and unwrap the result
    /// tile_map.set_tile(Vec3::new(15., 15., 0.), Tile::new(1)).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the given coordinate or index is out of bounds.
    pub fn set_tile<I: ToIndex + ToCoord3>(&mut self, v: I, tile: Tile) -> DimensionResult<()> {
        let coord = v.to_coord3(self.dimensions.width(), self.dimensions.height());
        let chunk_coord = self.tile_coord_to_chunk_coord(coord);
        let chunk_index = chunk_coord.to_index(self.dimensions.width(), self.dimensions.height());
        self.dimensions.check_index(chunk_index)?;

        let tile_y = coord.y() / self.chunk_dimensions.height();
        let map_coord = Vec2::new(
            coord.x() / self.chunk_dimensions.width(),
            self.dimensions.height() - (self.dimensions.max_y() as f32 - tile_y),
        );
        let x = coord.x() - (map_coord.x() * self.chunk_dimensions.width());
        let y = coord.y() - tile_y * self.chunk_dimensions.height();
        let coord = Vec3::new(x, y, coord.z());
        self.chunk_dimensions.check_coord(&coord)?;

        let mut setter = TileSetter::with_capacity(1);
        setter.push(coord, tile);
        self.events.send(MapEvent::Modified {
            index: chunk_index,
            setter,
        });
        Ok(())
    }

    /// Sets many tiles using a `TileSetter`.
    ///
    /// If setting a single tile is more preferable, then use the [set_tile]
    /// method instead.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::TileMap;
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
    /// use bevy_tilemap::Tile;
    /// use bevy_tilemap::tile::TileSetter;
    ///
    /// // Add a chunk
    /// tile_map.new_chunk(0);
    ///
    /// let mut new_tiles = TileSetter::new();
    /// new_tiles.push(Vec3::new(1., 1., 0.), Tile::new(1));
    /// new_tiles.push(Vec3::new(2., 2., 0.), Tile::new(2));
    /// new_tiles.push(Vec3::new(3., 3., 0.), Tile::new(3));
    ///
    /// // Set multiple tiles and unwrap the result
    /// tile_map.set_tiles(new_tiles).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the given coordinate or index is out of bounds.
    pub fn set_tiles(&mut self, setter: TileSetter) -> DimensionResult<()> {
        let mut tiles_map: HashMap<usize, TileSetter> = HashMap::default();
        for (setter_coord, setter_tile) in setter.iter() {
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
                setters.push(coord, *setter_tile);
            } else {
                let mut setter = TileSetter::new();
                setter.push(coord, *setter_tile);
                tiles_map.insert(chunk_index, setter);
            }
        }

        for (index, setter) in tiles_map {
            self.events.send(MapEvent::Modified { index, setter })
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
    /// # use bevy_tilemap::TileMap;
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
    /// use bevy_tilemap::TileMap;
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
    pub fn chunk_size(&self) -> Vec2 {
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

/// A component bundle for `TileMap` entities.
#[derive(Bundle, Debug)]
pub struct TileMapComponents {
    /// A `TileMap` which maintains chunks and its tiles.
    pub tile_map: TileMap,
    /// The transform location in a space for a component.
    pub transform: Transform,
    /// The global transform location in a space for a component.
    pub global_transform: GlobalTransform,
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
        let mut refresh_chunks = HashSet::<Handle<Chunk>>::default();
        let mut modified_chunks = Vec::new();
        let mut despawned_chunks = HashSet::<(Handle<Chunk>, Entity)>::default();
        let mut removed_chunks = HashSet::<(usize, Entity)>::default();
        let mut reader = map.events.get_reader();
        for event in reader.iter(&map.events) {
            use MapEvent::*;
            match event {
                Created { index, tiles } => {
                    new_chunks.push((*index, tiles.clone()));
                }
                Refresh { ref handle } => {
                    refresh_chunks.insert(handle.clone_weak());
                }
                Modified { index, setter } => {
                    modified_chunks.push((*index, setter.clone()));
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
        for (idx, tiles) in new_chunks.iter() {
            let (tile_indexes, tile_colors) = crate::tile::tiles_to_renderer_parts(tiles);

            let mut mesh = Mesh::from(ChunkMesh::new(map.chunk_dimensions));
            mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, tile_indexes.into());
            mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, tile_colors.into());
            let mesh_handle = meshes.add(mesh);

            let chunk = Chunk::new(tiles.clone(), mesh_handle.clone());
            let chunk_handle = chunks.add(chunk);
            map.chunks[*idx] = Some(chunk_handle);

            let map_coord = map.dimensions().decode_coord_unchecked(*idx);
            let map_center = map.dimensions().center();

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
                    texture_atlas: map.texture_atlas().clone_weak(),
                    chunk_dimensions: map.chunk_dimensions().into(),
                    mesh: mesh_handle.clone_weak(),
                    transform: Transform::from_translation(translation),
                    ..Default::default()
                })
                .current_entity()
                .expect("Chunk entity unexpected does not exist.");
            chunk_entities.push(chunk_entity);
        }
        commands.push_children(map_entity, &chunk_entities);

        for (index, setter) in modified_chunks.iter() {
            let chunk_handle = map.chunks[*index].as_ref().unwrap();
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
            map.chunks[*index] = None;
            commands.despawn(*entity);
        }
    }
}
