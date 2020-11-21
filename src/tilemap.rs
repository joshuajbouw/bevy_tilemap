use crate::{
    chunk::{Chunk, LayerKind},
    dimension::{Dimension2, DimensionError},
    entity::ChunkComponents,
    lib::*,
    mesh::ChunkMesh,
    point::{Point2, Point3},
    tile::{Tile, TilePoints, Tiles},
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// The kinds of errors that can occur.
pub(crate) enum ErrorKind {
    /// If the coordinate or index is out of bounds.
    DimensionError(DimensionError),
    /// If a layer already exists this error is returned.
    LayerExists(usize),
    /// If a layer does not already exist this error is returned.
    LayerDoesNotExist(usize),
    /// Texture atlas was not set
    MissingTextureAtlas,
    /// Chunk does not exist at coordinate.
    ChunkDoesNotExist(Point2),
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use ErrorKind::*;
        match self {
            DimensionError(err) => ::std::fmt::Debug::fmt(&err, f),
            LayerExists(n) => write!(
                f,
                "layer {} already exists, try `remove_layer` or `move_layer` first",
                n
            ),
            LayerDoesNotExist(n) => write!(f, "layer {} does not exist, try `add_layer` first", n),
            MissingTextureAtlas => write!(
                f,
                "texture atlas is missing, must use `TilemapBuilder::texture_atlas`"
            ),
            ChunkDoesNotExist(p) => write!(f, "chunk {} does not exist, try `add_chunk` first", p),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// The error type for operations when interacting with the tilemap.
pub struct TilemapError(Box<ErrorKind>);

impl Display for TilemapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Error for TilemapError {}

impl From<ErrorKind> for TilemapError {
    fn from(kind: ErrorKind) -> TilemapError {
        TilemapError(Box::new(kind))
    }
}

impl From<DimensionError> for TilemapError {
    fn from(err: DimensionError) -> TilemapError {
        TilemapError(Box::new(ErrorKind::DimensionError(err)))
    }
}

/// A map result.
pub type TilemapResult<T> = Result<T, TilemapError>;

/// Events that happen on a chunk by index value.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum TilemapEvent {
    /// To be used when a chunk is created.
    CreatedChunk {
        /// The index of the chunk.
        point: Point2,
        /// The Handle of the Chunk.
        handle: Handle<Chunk>,
    },
    /// An event when a layer is created for all chunks.
    AddedLayer {
        /// The *Z* layer to add.
        z_layer: usize,
        /// The `LayerKind` of the layer.
        kind: LayerKind,
    },
    /// An event when a layer is moved.
    MovedLayer {
        /// From which *Z* layer.
        from_z_layer: usize,
        /// To which *Z* layer.
        to_z_layer: usize,
    },
    /// An event when a layer is removed for all chunks.
    RemovedLayer {
        /// The *Z* layer to remove.
        z_layer: usize,
    },
    /// An event when a chunk had been modified by changing tiles.
    ModifiedChunk {
        /// The map index where the chunk needs to be stored.
        handle: Handle<Chunk>,
        /// The `TileSetter` that is used to set all the tiles.
        tiles: TilePoints,
    },
    /// An event when a chunk is spawned.
    SpawnedChunk { handle: Handle<Chunk> },
    /// If the chunk needs to be despawned, this event is used.
    DespawnedChunk {
        /// The handle of the chunk that needs to be despawned.
        handle: Handle<Chunk>,
    },
    /// If the chunk needs to be removed.
    RemovedChunk {
        /// The handle of the chunk that needs to be removed.
        handle: Handle<Chunk>,
    },
}

/// A Tilemap which maintains chunks and its tiles within.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Tilemap {
    dimensions: Option<Dimension2>,
    chunk_dimensions: Dimension2,
    tile_dimensions: Dimension2,
    layers: Vec<Option<LayerKind>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    texture_atlas: Handle<TextureAtlas>,
    #[cfg_attr(feature = "serde", serde(skip))]
    chunks: HashMap<Point2, Handle<Chunk>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    entities: HashMap<usize, Vec<Entity>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    events: Events<TilemapEvent>,
}

/// Tilemap factory, which can be used to construct and configure new tilemaps.
///
/// Methods can be chained in order to configure it. The [`texture_atlas`]
/// method is **required** in order to have a successful factory creation.
///
/// The configuration options available are:
///
/// - [`dimensions`]: specifies the dimensions of the tilemap. If this
/// is not set, then the tilemap will have no dimensions.
/// - [`chunk_dimensions`]: specifies the chunk's dimensions in tiles.
/// Default is 32x, 32y.
/// - [`tile_dimensions`]: specifies the tile's dimensions in pixels.
/// Default is 32px, 32px.
/// - [`z_layers`]: specifies the maximum number of layers that sprites
/// can exist on. Default is 20.
/// - [`texture_atlas`]: specifies the texture atlas handle
/// to use for the tilemap.
///
/// The [`build`] method will take ownership and consume the builder returning
/// a [`TilemapResult`] with either an [`TilemapError`] or the [tilemap].
///
/// # Examples
/// ```
/// use bevy_tilemap::tilemap;
/// use bevy::asset::HandleId;
/// use bevy::prelude::*;
///
/// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
///
/// let builder = tilemap::Builder::new().texture_atlas(texture_atlas_handle);
///
/// let tilemap = builder.build().unwrap();
/// ```
///
/// Can also get a builder like this:
/// ```
/// use bevy_tilemap::tilemap::Tilemap;
/// use bevy::asset::HandleId;
/// use bevy::prelude::*;
///
/// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
///
/// let builder = Tilemap::builder().texture_atlas(texture_atlas_handle);
///
/// let tilemap = builder.build().unwrap();
/// ```
///
/// [`build`]: Builder::build
/// [`chunk_dimensions`]: Builder::chunk_dimensions
/// [`dimensions`]: Builder::dimensions
/// [`texture_atlas`]: Builder::texture_atlas
/// [`tile_dimensions`]: Builder::tile_dimensions
/// [`z_layers`]: Builder::z_layers
/// [tilemap]: Tilemap
/// [`TilemapError`]: TilemapError
/// [`TilemapResult`]: TilemapResult
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Builder {
    dimensions: Option<Dimension2>,
    chunk_dimensions: Dimension2,
    tile_dimensions: Dimension2,
    z_layers: usize,
    layers: Option<HashMap<usize, LayerKind>>,
    texture_atlas: Option<Handle<TextureAtlas>>,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            dimensions: None,
            chunk_dimensions: Dimension2::new(32, 32),
            tile_dimensions: Dimension2::new(32, 32),
            z_layers: 20,
            layers: None,
            texture_atlas: None,
        }
    }
}

impl Builder {
    /// Configures the builder with the default settings.
    ///
    /// Is equivalent to [`default`] and [`builder`] method in the
    /// [tilemap]. Start with this then you are able to method chain.
    ///
    /// [`default`]: Builder::default
    /// [`builder`]: Tilemap::builder
    /// [tilemap]: Tilemap
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap::{self, Tilemap};
    ///
    /// let builder = tilemap::Builder::new();
    ///
    /// // Equivalent to...
    ///
    /// let builder = tilemap::Builder::default();
    ///
    /// // Or...
    ///
    /// let builder = Tilemap::builder();
    /// ```
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Sets the dimensions of the tilemap.
    ///
    /// If this is not set then the tilemap will be boundless entirely.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    ///
    /// let builder = tilemap::Builder::new().dimensions(5, 5);
    /// ```
    pub fn dimensions(mut self, width: u32, height: u32) -> Builder {
        self.dimensions = Some(Dimension2::new(width, height));
        self
    }

    /// Sets the chunk dimensions.
    ///
    /// Chunk dimensions are in tiles. If this is not set then the default of
    /// 32x, 32y is used.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    ///
    /// let builder = tilemap::Builder::new().chunk_dimensions(32, 32);
    /// ```
    pub fn chunk_dimensions(mut self, width: u32, height: u32) -> Builder {
        self.chunk_dimensions = Dimension2::new(width, height);
        self
    }

    /// Sets the tile dimensions.
    ///
    /// Tile dimensions are in pixels. If this is not set then the default of
    /// 32px, 32px is used.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    ///
    /// let builder = tilemap::Builder::new().tile_dimensions(32, 32);
    /// ```
    pub fn tile_dimensions(mut self, width: u32, height: u32) -> Builder {
        self.tile_dimensions = Dimension2::new(width, height);
        self
    }

    /// Sets the amount of render layers that sprites can exist on.
    ///
    /// By default there are 20 if this is not set.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    ///
    /// let builder = tilemap::Builder::new().z_layers(5);
    /// ```
    pub fn z_layers(mut self, layers: usize) -> Builder {
        self.z_layers = layers;
        self
    }

    /// Adds a sprite layer that sprites can exist on.
    ///
    /// Takes in a [`LayerKind`] and a Z layer and adds it to the builder.
    ///
    /// If there are more layers than Z layers is set, builder will construct
    /// a tilemap with that many layers instead. In the case that a layer is
    /// added twice to the same Z layer, the first layer will be overwritten by
    /// the later.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy_tilemap::chunk::LayerKind;
    ///
    /// let builder = tilemap::Builder::new()
    ///     .add_layer(LayerKind::Dense, 0)
    ///     .add_layer(LayerKind::Sparse, 1)
    ///     .add_layer(LayerKind::Sparse, 2);
    /// ```
    ///
    /// [`LayerKind`]: crate::chunk::LayerKind
    pub fn add_layer(mut self, kind: LayerKind, z_layer: usize) -> Builder {
        if let Some(layers) = &mut self.layers {
            layers.insert(z_layer, kind);
        } else {
            let mut layers = HashMap::default();
            layers.insert(z_layer, kind);
            self.layers = Some(layers);
        }
        self
    }

    /// Sets the texture atlas, this is **required** to be set.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let builder = tilemap::Builder::new().texture_atlas(texture_atlas_handle);
    /// ```
    pub fn texture_atlas(mut self, handle: Handle<TextureAtlas>) -> Builder {
        self.texture_atlas = Some(handle);
        self
    }

    /// Consumes the builder and returns a result.
    ///
    /// If successful a [`TilemapResult`] is return with [tilemap] on
    /// succes or a [`TilemapError`] if there is an issue.
    ///
    /// # Errors
    /// If a texture atlas is not set this is the only way that an error can
    /// occur. If this happens, be sure to use [`texture_atlas`].
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let builder = tilemap::Builder::new().texture_atlas(texture_atlas_handle);
    ///
    /// let tilemap = builder.build();
    /// ```
    ///
    /// [`texture_atlas`]: Builder::texture_atlas
    /// [tilemap]: Tilemap
    /// [`TilemapError`]: TilemapError
    /// [`TilemapResult`]: TilemapResult
    pub fn build(self) -> TilemapResult<Tilemap> {
        let texture_atlas = if let Some(atlas) = self.texture_atlas {
            atlas
        } else {
            return Err(ErrorKind::MissingTextureAtlas.into());
        };

        let z_layers = if let Some(layers) = &self.layers {
            if self.z_layers > layers.len() {
                self.z_layers
            } else {
                layers.len()
            }
        } else {
            self.z_layers
        };

        let mut tilemap = Tilemap {
            dimensions: self.dimensions,
            chunk_dimensions: self.chunk_dimensions,
            tile_dimensions: self.tile_dimensions,
            layers: vec![None; z_layers],
            texture_atlas,
            chunks: Default::default(),
            entities: Default::default(),
            events: Default::default(),
        };

        if let Some(mut layers) = self.layers {
            for (z_layer, kind) in layers.drain() {
                tilemap
                    .add_layer_with_kind(kind, z_layer)
                    .expect("A layer already exists here.");
            }
        }

        Ok(tilemap)
    }
}

impl TypeUuid for Tilemap {
    const TYPE_UUID: Uuid = Uuid::from_u128(109481186966523254410691740507722642628);
}

impl Default for Tilemap {
    fn default() -> Self {
        Tilemap {
            dimensions: None,
            chunk_dimensions: Dimension2::new(32, 32),
            tile_dimensions: Dimension2::new(32, 32),
            layers: vec![None; 20],
            texture_atlas: Handle::default(),
            chunks: Default::default(),
            entities: Default::default(),
            events: Default::default(),
        }
    }
}

impl Tilemap {
    /// Constructs a new Tilemap with the required texture atlas and default
    /// configuration.
    ///
    /// This differs from [`default`] in that it requires the texture atlas
    /// handle.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap::Tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = Tilemap::new(texture_atlas_handle);
    /// ```
    ///
    /// [`default`]: Tilemap::default
    pub fn new(texture_atlas: Handle<TextureAtlas>) -> Tilemap {
        Tilemap {
            texture_atlas,
            ..Default::default()
        }
    }

    /// Configures the builder with the default settings.
    ///
    /// Is equivalent to [`default`] and [`builder`] method in the
    /// [tilemap]. Start with this then you are able to method chain.
    ///
    /// [`default`]: Builder::default
    /// [`builder`]: Tilemap::builder
    /// [tilemap]: Tilemap
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap::{self, Tilemap};
    ///
    /// let builder = tilemap::Builder::new();
    ///
    /// // Equivalent to...
    ///
    /// let builder = tilemap::Builder::default();
    ///
    /// // Or...
    ///
    /// let builder = Tilemap::builder();
    /// ```
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Sets the sprite sheet for use in the tilemap.
    ///
    /// This can be used if the need to swap the sprite sheet for another is
    /// wanted.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # let mut tilemap = Tilemap::default();
    /// #
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// tilemap.set_texture_atlas(texture_atlas_handle);
    /// ```
    pub fn set_texture_atlas(&mut self, handle: Handle<TextureAtlas>) {
        self.texture_atlas = handle;
    }

    /// Returns a reference of the handle of the texture atlas.
    ///
    /// The Handle is used to get the correct sprite sheet that is used for this
    /// tilemap with the renderer.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let tilemap = Tilemap::new(texture_atlas_handle);
    /// let texture_atlas: &Handle<TextureAtlas> = tilemap.texture_atlas();
    /// ```
    pub fn texture_atlas(&self) -> &Handle<TextureAtlas> {
        &self.texture_atlas
    }

    fn new_chunk_with_handle(&mut self, point: Point2, handle: Handle<Chunk>) {
        self.chunks.insert(point, handle.clone_weak());

        self.events
            .send(TilemapEvent::CreatedChunk { point, handle });
    }

    /// Constructs a new chunk and stores it at a coordinate position.
    ///
    /// It requires that you give it either a point. It then automatically sets
    /// both a sized mesh and chunk for use based on the parameters set in the
    /// parent tilemap.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// // Add some chunks.
    /// tilemap.new_chunk((0, 0)).unwrap();
    /// tilemap.new_chunk((1, 1)).unwrap();
    /// tilemap.new_chunk((2, 2)).unwrap();
    /// ```
    pub fn new_chunk<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point: Point2 = point.into();
        if let Some(dimensions) = &self.dimensions {
            dimensions.check_point(point)?;
        }

        let handle: Handle<Chunk> = Handle::weak(HandleId::random::<Chunk>());
        self.new_chunk_with_handle(point, handle);
        Ok(())
    }

    /// Adds a layer to the tilemap with a specified layer kind.
    ///
    /// This method takes in a [`LayerKind`] as well as a specified Z layer
    /// that the layer needs to be set to. If the layer is already the specified
    /// layer's kind, then nothing happens.
    ///
    /// # Errors
    ///
    /// If a layer is set and a different layer already exists at that Z layer
    /// then an error is returned regarding that. This is done to prevent
    /// accidental overwrites of a layer.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// use bevy_tilemap::chunk::LayerKind;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    ///
    /// let kind = LayerKind::Sparse;
    ///
    /// tilemap.add_layer_with_kind(kind, 1).unwrap();
    /// ```
    ///
    /// [`LayerKind`]: crate::chunk::LayerKind
    pub fn add_layer_with_kind(&mut self, kind: LayerKind, z_layer: usize) -> TilemapResult<()> {
        if let Some(some_kind) = self.layers[z_layer] {
            return if some_kind == kind {
                Ok(())
            } else {
                Err(ErrorKind::LayerExists(z_layer).into())
            };
        }

        self.layers[z_layer] = Some(kind);

        self.events.send(TilemapEvent::AddedLayer { z_layer, kind });
        Ok(())
    }

    /// Adds a layer to the tilemap.
    ///
    /// This method creates a layer across all chunks at the specified Z layer.
    /// For ease of use, it by default makes a layer with a dense
    /// [`LayerKind`] which is ideal for layers full of sprites.
    ///
    /// If you want to use a layer that is more performant and less data heavy,
    /// use [`add_layer_with_kind`] with [`LayerKind::Sparse`].
    ///
    /// If the layer is already the specified layer's kind, then nothing
    /// happens.
    ///
    /// # Errors
    ///
    /// If a layer is set and a different layer already exists at that Z layer
    /// then an error is returned regarding that. This is done to prevent
    /// accidental overwrites of a layer.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// # use bevy_tilemap::chunk::LayerKind;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// tilemap.add_layer(1).unwrap();
    /// ```
    ///
    /// [`add_layer_with_kind`]: Tilemap::add_layer_with_kind
    /// [`LayerKind`]: crate::chunk::LayerKind;
    /// [`LayerKind::Sparse`]: crate::chunk::LayerKind::Sparse;
    pub fn add_layer(&mut self, z_layer: usize) -> TilemapResult<()> {
        self.add_layer_with_kind(LayerKind::Dense, z_layer)
    }

    /// Moves a layer from one Z level to another.
    ///
    /// # Errors
    ///
    /// If the destination exists, it will throw an error. Likewise, if the
    /// origin does not exist, it also will throw an error.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// tilemap.add_layer(0).unwrap();
    /// tilemap.add_layer(3).unwrap();
    ///
    /// // If we moved this to layer 3, it would instead fail.
    /// tilemap.move_layer(0, 2).unwrap();
    /// ```
    pub fn move_layer(&mut self, from_z: usize, to_z: usize) -> TilemapResult<()> {
        if self.layers[to_z].is_some() {
            return Err(ErrorKind::LayerExists(to_z).into());
        };

        if self.layers[from_z].is_none() {
            return Err(ErrorKind::LayerDoesNotExist(from_z).into());
        };

        self.layers[to_z] = self.layers[from_z];
        self.layers[from_z] = None;

        self.events.send(TilemapEvent::MovedLayer {
            from_z_layer: from_z,
            to_z_layer: to_z,
        });

        Ok(())
    }

    /// Removes a layer from the tilemap and inner chunks.
    ///
    /// **Warning**: This is destructive if you have tiles that exist on that
    /// layer. If you want to add them back in, better to use the [`move_layer`]
    /// method instead.
    ///
    /// This method takes in a Z layer which is then flagged for deletion. If
    /// the layer already does not exist, it does nothing.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// // This sends a create layer event.
    /// tilemap.add_layer(1);
    ///
    /// // And this sends a removed layer event which will prevent it from
    /// // existing between frames.
    /// tilemap.remove_layer(1);
    /// ```
    ///
    /// [`move_layer`]: TIlemap::move_layer
    pub fn remove_layer(&mut self, z: usize) {
        if self.layers[z].is_none() {
            return;
        }

        self.layers[z] = None;

        self.events.send(TilemapEvent::RemovedLayer { z_layer: z })
    }

    /// Spawns a chunk at a given index or coordinate.
    ///
    /// # Errors
    ///
    /// If the coordinate or index is out of bounds or if the chunk does not
    /// exist, an error will be returned.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// tilemap.new_chunk((0, 0));
    ///
    /// // Ideally you should want to set some tiles here else nothing will
    /// // display in the render...
    ///
    /// tilemap.spawn_chunk((0, 0));
    /// ```
    pub fn spawn_chunk<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point: Point2 = point.into();
        if let Some(dimensions) = &self.dimensions {
            dimensions.check_point(point)?;
        }
        if !self.chunks.contains_key(&point) {
            return Err(ErrorKind::ChunkDoesNotExist(point).into());
        }

        let handle = self.chunks.get(&point).expect("`Chunk` is missing.");

        self.events.send(TilemapEvent::SpawnedChunk {
            handle: handle.clone_weak(),
        });

        Ok(())
    }

    /// Spawns a chunk at a given tile point.
    ///
    /// # Errors
    ///
    /// If the coordinate or index is out of bounds or if the chunk does not
    /// exist, an error will be returned.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// let tile_point = (16, 16, 0);
    /// let tile_index = 0;
    /// tilemap.set_tile(tile_point, tile_index);
    ///
    /// tilemap.spawn_chunk_containing_point(tile_point);
    /// ```
    pub fn spawn_chunk_containing_point<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point = self.tile_to_chunk_point(point);
        self.spawn_chunk(point)
    }

    /// De-spawns a spawned chunk at a given index or coordinate.
    ///
    /// If the chunk is not spawned this will result in nothing.
    ///
    /// # Errors
    ///
    /// If the coordinate or index is out of bounds, an error will be returned.
    /// Also if the chunk that needs to spawn does not exist expect an error.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// tilemap.new_chunk((0, 0)).unwrap();
    ///
    /// // Ideally you should want to set some tiles here else nothing will
    /// // display in the render...
    ///
    /// tilemap.spawn_chunk((0, 0)).unwrap();
    ///
    /// // Later a frame or more on...
    ///
    /// tilemap.despawn_chunk((0, 0)).unwrap();
    /// ```
    pub fn despawn_chunk<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point: Point2 = point.into();
        if let Some(dimensions) = &self.dimensions {
            dimensions.check_point(point)?;
        }
        if !self.chunks.contains_key(&point) {
            return Err(ErrorKind::ChunkDoesNotExist(point).into());
        }

        let handle = self.chunks.get(&point).expect("`Chunk` is missing.");

        self.events.send(TilemapEvent::DespawnedChunk {
            handle: handle.clone_weak(),
        });

        Ok(())
    }

    /// Destructively removes a chunk at a coordinate position.
    ///
    /// Internally, this sends an event to the tilemap's system flagging which
    /// chunks must be removed by index and entity. A chunk is not recoverable
    /// if this action is done.
    ///
    /// # Errors
    ///
    /// If the coordinate or index is out of bounds, an error will be returned.
    /// Also if the chunk that needs to spawn does not exist expect an error.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// // Add some chunks.
    /// tilemap.new_chunk((0, 0)).unwrap();
    /// tilemap.new_chunk((1, 1)).unwrap();
    /// tilemap.new_chunk((2, 2)).unwrap();
    ///
    /// // Remove the same chunks in the same frame. Do note that adding then
    /// // removing in the same frame will prevent the entity from spawning at
    /// // all.
    /// tilemap.remove_chunk((0, 0)).unwrap();
    /// tilemap.remove_chunk((1, 1)).unwrap();
    /// tilemap.remove_chunk((2, 2)).unwrap();
    /// ```
    pub fn remove_chunk<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point: Point2 = point.into();
        if let Some(dimensions) = &self.dimensions {
            dimensions.check_point(point)?;
        }
        if !self.chunks.contains_key(&point) {
            return Err(ErrorKind::ChunkDoesNotExist(point).into());
        }

        let handle = self.chunks.get(&point).expect("`Chunk` is missing.");

        self.events.send(TilemapEvent::RemovedChunk {
            handle: handle.clone_weak(),
        });

        Ok(())
    }

    /// Takes a tile point and changes it into a chunk point.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// let tile_point = (15, 15);
    /// let chunk_point = tilemap.tile_to_chunk_point(tile_point);
    ///
    /// assert_eq!((0, 0), chunk_point);
    ///
    /// let tile_point = (16, 16);
    /// let chunk_point = tilemap.tile_to_chunk_point(tile_point);
    ///
    /// assert_eq!((1, 1), chunk_point);
    ///
    /// let tile_point = (-16, -16);
    /// let chunk_point = tilemap.tile_to_chunk_point(tile_point);
    ///
    /// assert_eq!((-0, -0), chunk_point);
    ///
    /// let tile_point = (-17, 17);
    /// let chunk_point = tilemap.tile_to_chunk_point(tile_point);
    /// assert_eq!((-1, -1), chunk_point);
    /// ```
    pub fn tile_to_chunk_point<P: Into<Point2>>(&self, point: P) -> (i32, i32) {
        let point: Point2 = point.into();
        let width = self.chunk_dimensions.width() as f32;
        let height = self.chunk_dimensions.height() as f32;
        let x = ((point.x as f32 + width / 2.0) / width).floor() as i32;
        let y = ((point.y as f32 + height / 2.0) / height).floor() as i32;
        (x, y)
    }

    /// Sets many tiles, creating new chunks if needed.
    ///
    /// If setting a single tile is more preferable, then use the [`set_tile`]
    /// method instead.
    ///
    /// If the chunk does not yet exist, it will create a new one automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if the given coordinate or index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// use bevy_tilemap::tile::{Tile, Tiles};
    ///
    /// let mut tiles = Tiles::default();
    /// tiles.insert((1, 1, 0), Tile::new(1));
    /// tiles.insert((2, 2, 0), Tile::new(2));
    /// tiles.insert((3, 3, 0), Tile::new(3));
    ///
    /// assert_eq!(tiles.len(), 3);
    ///
    /// // Set multiple tiles and unwrap the result
    /// tilemap.set_tiles(tiles).unwrap();
    /// ```
    ///
    /// [`set_tile`]: Tilemap::set_tile
    pub fn set_tiles<T>(&mut self, tiles: T) -> TilemapResult<()>
    where
        T: IntoIterator<Item = ((i32, i32, i32), Tile)>,
    {
        let width = self.chunk_dimensions.width() as i32;
        let height = self.chunk_dimensions.height() as i32;

        let mut chunk_map: HashMap<Point2, TilePoints> = HashMap::default();
        for (points, tile) in tiles.into_iter() {
            let global_tile_point: Point3 = points.into();
            let chunk_point: Point2 = self.tile_to_chunk_point(&global_tile_point).into();

            if self.layers[global_tile_point.z as usize].is_none() {
                self.add_layer(global_tile_point.z as usize)?;
            }

            let tile_point = Point3::new(
                global_tile_point.x - (width * chunk_point.x) + (width / 2),
                global_tile_point.y - (height * chunk_point.y) + (width / 2),
                global_tile_point.z,
            );
            if let Some(tiles) = chunk_map.get_mut(&chunk_point) {
                tiles.insert(tile_point, tile);
            } else {
                let mut tiles = TilePoints::default();
                tiles.insert(tile_point, tile);
                chunk_map.insert(chunk_point, tiles);
            }
        }

        for (chunk_point, tiles) in chunk_map {
            let handle = if let Some(handle) = self.chunks.get(&chunk_point) {
                handle.clone_weak()
            } else {
                let handle: Handle<Chunk> = Handle::weak(HandleId::random::<Chunk>());
                self.new_chunk_with_handle(chunk_point, handle.clone_weak());
                handle
            };

            self.events
                .send(TilemapEvent::ModifiedChunk { handle, tiles })
        }
        Ok(())
    }

    /// Sets a single tile at a coordinate position, creating a chunk if necessary.
    ///
    /// For convenience, this does not require to use a TileSetter which is beneficial for multiple
    /// tiles. If that is preferred, do use [`set_tiles`] instead.
    ///
    /// If the chunk does not yet exist, it will create a new one automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if the given coordinate or index is out of bounds.
    ///
    /// # Examples
    /// ```
    /// # use bevy_tilemap::tilemap::Tilemap;
    /// # use bevy::asset::HandleId;
    /// # use bevy::prelude::*;
    /// #
    /// # // In production use a strong handle from an actual source.
    /// # let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    /// #
    /// # let mut tilemap = Tilemap::new(texture_atlas_handle);
    /// #
    /// use bevy_tilemap::tile::Tile;
    ///
    /// // Set a single tile and unwrap the result
    /// tilemap.set_tile((15, 15, 0), Tile::new(1)).unwrap();
    /// ```
    ///
    /// [`set_tiles`]: Tilemap::set_tiles
    pub fn set_tile<P, T>(&mut self, point: P, tile: T) -> TilemapResult<()>
    where
        P: Into<Point3>,
        T: Into<Tile>,
    {
        let tile: Tile = tile.into();
        let mut tiles = Tiles::default();
        let point: Point3 = point.into();
        tiles.insert((point.x, point.y, point.z), tile);
        self.set_tiles(tiles)
    }

    /// Returns the center tile, if the tilemap has dimensions.
    ///
    /// Returns `None` if the tilemap has no constrainted dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = tilemap::Builder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .dimensions(32, 32)
    ///     .build()
    ///     .unwrap();
    ///
    /// let center: (i32, i32) = tilemap.center_tile_coord().unwrap();
    ///
    /// // 32 * 32 / 2 = 512
    /// assert_eq!((512, 512), center);
    /// ```
    pub fn center_tile_coord(&self) -> Option<(i32, i32)> {
        self.dimensions.map(|dimensions| {
            (
                (dimensions.width() / 2 * self.chunk_dimensions.width()) as i32,
                (dimensions.height() / 2 * self.chunk_dimensions.height()) as i32,
            )
        })
    }

    /// The width of the tilemap in chunks, if it has dimensions.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = tilemap::Builder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .dimensions(32, 32)
    ///     .build()
    ///     .unwrap();
    ///
    /// let width: u32 = tilemap.width().unwrap();
    ///
    /// assert_eq!(width, 32);
    /// ```
    pub fn width(&self) -> Option<u32> {
        self.dimensions.map(|dimensions| dimensions.width())
    }

    /// The height of the tilemap in chunks, if it has dimensions.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = tilemap::Builder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .dimensions(32, 32)
    ///     .build()
    ///     .unwrap();
    ///
    /// let height: u32 = tilemap.height().unwrap();
    ///
    /// assert_eq!(height, 32);
    /// ```
    pub fn height(&self) -> Option<u32> {
        self.dimensions.map(|dimensions| dimensions.height())
    }

    /// The width of all the chunks in tiles.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = tilemap::Builder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .chunk_dimensions(32, 32)
    ///     .build()
    ///     .unwrap();
    ///
    /// let chunk_width: u32 = tilemap.chunk_width();
    ///
    /// assert_eq!(chunk_width, 32);
    /// ```
    pub fn chunk_width(&self) -> u32 {
        self.chunk_dimensions.width()
    }

    /// The height of all the chunks in tiles.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = tilemap::Builder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .chunk_dimensions(32, 32)
    ///     .build()
    ///     .unwrap();
    ///
    /// let chunk_height: u32 = tilemap.chunk_height();
    ///
    /// assert_eq!(chunk_height, 32);
    /// ```
    pub fn chunk_height(&self) -> u32 {
        self.chunk_dimensions.height()
    }

    /// The width of a tile in pixels.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = tilemap::Builder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .tile_dimensions(32, 32)
    ///     .build()
    ///     .unwrap();
    ///
    /// let tile_width: u32 = tilemap.tile_width();
    ///
    /// assert_eq!(tile_width, 32);
    /// ```
    pub fn tile_width(&self) -> u32 {
        self.tile_dimensions.width()
    }

    /// The height of a tile in pixels.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::tilemap;
    /// use bevy::asset::HandleId;
    /// use bevy::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = tilemap::Builder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .tile_dimensions(32, 32)
    ///     .build()
    ///     .unwrap();
    ///
    /// let tile_height: u32 = tilemap.tile_height();
    ///
    /// assert_eq!(tile_height, 32);
    /// ```
    pub fn tile_height(&self) -> u32 {
        self.tile_dimensions.height()
    }
}

/// The event handling system for the tilemap.
///
/// There are a few things that happen in this function which are outlined in
/// order of operation here. It was done in this order that made the most sense
/// at the time of creation.
///
/// 1. Add new chunks
/// 1. Add new layers
/// 1. Move layers
/// 1. Remove layers
/// 1. Modify layers
/// 1. Spawn chunks
/// 1. Despawn chunks
/// 1. Remove chunks
#[inline(always)]
pub fn map_system(
    mut commands: Commands,
    mut chunks: ResMut<Assets<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &mut Tilemap)>,
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
            use TilemapEvent::*;
            match event {
                CreatedChunk {
                    ref point,
                    ref handle,
                } => {
                    new_chunks.push((*point, handle.clone_weak()));
                }
                AddedLayer {
                    z_layer: ref z,
                    ref kind,
                } => {
                    added_layers.push((*z, *kind));
                }
                MovedLayer {
                    from_z_layer: ref from_z,
                    to_z_layer: ref to_z,
                } => {
                    moved_layers.push((*from_z, *to_z));
                }
                RemovedLayer { z_layer: ref z } => {
                    removed_layers.push(*z);
                }
                ModifiedChunk {
                    ref handle,
                    tiles: ref setter,
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

        for (point, handle) in new_chunks.into_iter() {
            let chunk = Chunk::new(point, map.layers.len());
            let handle = chunks.set(handle.clone_weak(), chunk);
            map.chunks.insert(point, handle);
        }

        for (z, kind) in added_layers.into_iter() {
            for handle in map.chunks.values() {
                let chunk = chunks.get_mut(handle).expect("`Chunk` is missing.");
                chunk.add_layer(&kind, z, map.chunk_dimensions);
            }
        }

        for (from_z, to_z) in moved_layers.into_iter() {
            for handle in map.chunks.values() {
                let chunk = chunks.get_mut(handle).expect("`Chunk` is missing.");
                chunk.move_layer(from_z, to_z);
            }
        }

        for z in removed_layers.into_iter() {
            for handle in map.chunks.values() {
                let chunk = chunks.get_mut(handle).expect("`Chunk` is missing.");
                chunk.remove_layer(z);
            }
        }

        for (handle, setter) in modified_chunks.into_iter() {
            let chunk = chunks.get_mut(&handle).expect("`Chunk` is missing.");
            for (point, tile) in setter.into_iter() {
                let index = map.chunk_dimensions.encode_point_unchecked(point.xy());
                chunk.set_tile(point.z as usize, index, tile);
            }
        }

        let capacity = spawned_chunks.len();
        for handle in spawned_chunks.into_iter() {
            let chunk = chunks.get_mut(handle).expect("`Chunk` is missing.");
            let mut entities = Vec::with_capacity(capacity);
            for z in 0..map.layers.len() {
                let mut mesh = Mesh::from(&ChunkMesh::new(map.chunk_dimensions));
                let (indexes, colors) =
                    if let Some(parts) = chunk.tiles_to_renderer_parts(z, &map.chunk_dimensions) {
                        parts
                    } else {
                        continue;
                    };
                mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes.into());
                mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors.into());
                let mesh_handle = meshes.add(mesh);
                chunk.set_mesh(z, mesh_handle.clone());

                let translation = Vec3::new(
                    (chunk.point().x
                        * map.tile_dimensions.width() as i32
                        * map.chunk_dimensions.width() as i32) as f32,
                    (chunk.point().y
                        * map.tile_dimensions.height() as i32
                        * map.chunk_dimensions.height() as i32) as f32,
                    z as f32,
                );
                let entity = commands
                    .spawn(ChunkComponents {
                        texture_atlas: map.texture_atlas().clone_weak(),
                        chunk_dimensions: map.chunk_dimensions.into(),
                        mesh: mesh_handle.clone_weak(),
                        transform: Transform::from_translation(translation),
                        ..Default::default()
                    })
                    .current_entity()
                    .expect("Chunk entity unexpected does not exist.");

                chunk.add_entity(z, entity);
                entities.push(entity);
            }
            commands.push_children(map_entity, &entities);
        }

        for handle in despawned_chunks.into_iter() {
            let chunk = chunks.get_mut(handle).expect("`Chunk` is missing.");
            for entity in chunk.get_entities() {
                commands.despawn(entity);
            }
        }

        for handle in removed_chunks.into_iter() {
            let chunk = chunks.get_mut(handle).expect("`Chunk` is missing.");
            for entity in chunk.get_entities() {
                commands.despawn(entity);
            }
            map.chunks.remove(&chunk.point());
        }
    }
}
