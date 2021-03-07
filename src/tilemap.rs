//! # Constructing a basic tilemap, setting tiles, and spawning.
//!
//! Bevy Tilemap makes it easy to quickly implement a tilemap if you are in a
//! rush or want to build a conceptual game.
//!
//! ```
//! use bevy_asset::{prelude::*, HandleId};
//! use bevy_sprite::prelude::*;
//! use bevy_tilemap::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
//!
//! // Coordinate point with Z order.
//! let point = (16, 16);
//! let sprite_index = 0;
//! let tile = Tile { point: point.clone(), sprite_index, ..Default::default() };
//! tilemap.insert_tile(tile);
//!
//! tilemap.spawn_chunk_containing_point(point);
//! ```
//!
//! # Constructing a more advanced tilemap.
//!
//! For most cases, it is preferable to construct a tilemap with explicit
//! parameters. For that you would use a [`TilemapBuilder`].
//!
//! [`TilemapBuilder`]: crate::tilemap::TilemapBuilder
//!
//! ```
//! use bevy_asset::{prelude::*, HandleId};
//! use bevy_sprite::prelude::*;
//! use bevy_tilemap::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = TilemapBuilder::new()
//!     .texture_atlas(texture_atlas_handle)
//!     .chunk_dimensions(64, 64, 1)
//!     .texture_dimensions(8, 8)
//!     .dimensions(32, 32)
//!     .add_layer(TilemapLayer { kind: LayerKind::Dense, ..Default::default() }, 0)
//!     .add_layer(TilemapLayer { kind: LayerKind::Sparse, ..Default::default() }, 1)
//!     .add_layer(TilemapLayer { kind: LayerKind::Sparse, ..Default::default() }, 2)
//!     .z_layers(3)
//!     .finish();
//! ```
//!
//! The above example outlines all the current possible builder methods. What is
//! neat is that if more layers are accidentally set than z_layer set, it will
//! use the layer length instead. Much more features are planned including
//! automated systems that will enhance the tilemap further.
//!
//! # Setting tiles
//!
//! There are two methods to set tiles in the tilemap. The first is single tiles
//! at a time which is acceptable for tiny updates such as moving around
//! characters. The second being bulk setting many tiles at once.
//!
//! If you expect to move multiple tiles a frame, **always** use [`insert_tiles`].
//! A single event is created with all tiles if set this way.
//!
//! [`insert_tiles`]: crate::tilemap::Tilemap::insert_tiles
//!
//! ```
//! use bevy_asset::{prelude::*, HandleId};
//! use bevy_sprite::prelude::*;
//! use bevy_tilemap::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
//!
//! // Prefer this
//! let mut tiles = Vec::new();
//! for y in 0..31 {
//!     for x in 0..31 {
//!         tiles.push(Tile { point: (x, y), ..Default::default() });
//!     }
//! }
//!
//! tilemap.insert_tiles(tiles);
//!
//! // Over this...
//! for y in 0..31 {
//!     for x in 0..31 {
//!         tilemap.insert_tile(Tile { point: (x, y), ..Default::default() });
//!     }
//! }
//! ```

use crate::{
    chunk::{mesh::ChunkMesh, Chunk, LayerKind, RawTile},
    event::TilemapChunkEvent,
    lib::*,
    prelude::GridTopology,
    tile::Tile,
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// The kinds of errors that can occur.
pub enum ErrorKind {
    /// If the coordinate or index is out of bounds.
    DimensionError(DimensionError),
    /// If a layer already exists this error is returned.
    LayerExists(usize),
    /// If a layer does not already exist this error is returned.
    LayerDoesNotExist(usize),
    /// Texture atlas was not set
    MissingTextureAtlas,
    /// The tile dimensions were not set.
    MissingTextureDimensions,
    /// The chunk does not exist.
    MissingChunk,
    /// The chunk already exists.
    ChunkAlreadyExists(Point2),
}

impl Display for ErrorKind {
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
            MissingTextureDimensions => {
                write!(f, "tile dimensions are missing, it is required to set it")
            }
            MissingChunk => write!(f, "the chunk does not exist, try `add_chunk` first"),
            ChunkAlreadyExists(p) => write!(
                f,
                "the chunk {} already exists, if this was intentional run `remove_chunk` first",
                p
            ),
        }
    }
}

impl Error for ErrorKind {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// The error type for operations when interacting with the tilemap.
pub struct TilemapError(pub Box<ErrorKind>);

impl Display for TilemapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl Error for TilemapError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

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

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    struct AutoFlags: u16 {
        const NONE = 0b0;
        const AUTO_CONFIGURE = 0b0000_0000_0000_0001;
        const AUTO_CHUNK = 0b0000_0000_0000_0010;
        const AUTO_SPAWN = 0b0000_0000_0000_0100;
    }
}

/// The default texture dimensions in chunks.
const DEFAULT_TEXTURE_DIMENSIONS: Dimension2 = Dimension2::new(32, 32);
/// The default chunk dimensions in tiles.
const DEFAULT_CHUNK_DIMENSIONS: Dimension3 = Dimension3::new(32, 32, 1);
/// The default tile scale.
const DEFAULT_TILE_SCALE: (f32, f32, f32) = (1.0, 1.0, 1.0);
/// The default z layers.
const DEFAULT_Z_LAYERS: usize = 5;

impl Default for AutoFlags {
    fn default() -> Self {
        AutoFlags::AUTO_CONFIGURE & AutoFlags::AUTO_CHUNK
    }
}

/// A layer configuration for a tilemap.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TilemapLayer {
    /// The kind of layer to create.
    pub kind: LayerKind,
}

impl Default for TilemapLayer {
    fn default() -> TilemapLayer {
        TilemapLayer {
            kind: LayerKind::Dense,
        }
    }
}

/// A Tilemap which maintains chunks and its tiles within.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Tilemap {
    /// The type of grid to use.
    topology: GridTopology,
    /// An optional field which can contain the tilemaps dimensions in chunks.
    dimensions: Option<Dimension2>,
    /// A chunks dimensions in tiles.
    chunk_dimensions: Dimension3,
    /// The layer in the chunks offset value as X, Y. Each layer will be offset
    /// by this.
    layer_offset: Vec2,
    /// A mesh for a chunk which is stored here and copied when needed.
    chunk_mesh: ChunkMesh,
    /// A tiles dimensions in pixels.
    texture_dimensions: Dimension2,
    /// The layers that are currently set in the tilemap in order from lowest
    /// to highest.
    layers: Vec<Option<TilemapLayer>>,
    /// Auto flags used for different automated features.
    auto_flags: AutoFlags,
    /// Dimensions of chunks to spawn from camera transform.
    auto_spawn: Option<Dimension2>,
    /// Custom flags.
    custom_flags: Vec<u32>,
    #[cfg_attr(feature = "serde", serde(skip))]
    /// The handle of the texture atlas.
    texture_atlas: Handle<TextureAtlas>,
    /// A map of all the chunks at points.
    chunks: HashMap<Point2, Chunk>,
    #[cfg_attr(feature = "serde", serde(skip))]
    /// A map of all currently spawned entities.
    entities: HashMap<usize, Vec<Entity>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    /// The events of the tilemap.
    chunk_events: Events<TilemapChunkEvent>,
    /// A set of all spawned chunks.
    spawned: HashSet<(i32, i32)>,
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
/// The [`finish`] method will take ownership and consume the builder returning
/// a [`TilemapResult`] with either an [`TilemapError`] or the [tilemap].
///
/// # Examples
/// ```
/// use bevy_asset::{prelude::*, HandleId};
/// use bevy_sprite::prelude::*;
/// use bevy_tilemap::prelude::*;
///
/// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
///
/// let builder = TilemapBuilder::new().texture_dimensions(32, 32).texture_atlas(texture_atlas_handle);
///
/// let tilemap = builder.finish().unwrap();
/// ```
///
/// Can also get a builder like this:
/// ```
/// use bevy_asset::{prelude::*, HandleId};
/// use bevy_sprite::prelude::*;
/// use bevy_tilemap::prelude::*;
///
/// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
///
/// let builder = Tilemap::builder().texture_dimensions(32, 32).texture_atlas(texture_atlas_handle);
///
/// let tilemap = builder.finish().unwrap();
/// ```
///
/// [`finish`]: TilemapBuilder::finish
/// [`chunk_dimensions`]: TilemapBuilder::chunk_dimensions
/// [`dimensions`]: TilemapBuilder::dimensions
/// [`texture_atlas`]: TilemapBuilder::texture_atlas
/// [`tile_dimensions`]: TilemapBuilder::tile_dimensions
/// [`z_layers`]: TilemapBuilder::z_layers
/// [tilemap]: Tilemap
/// [`TilemapError`]: TilemapError
/// [`TilemapResult`]: TilemapResult
#[derive(Clone, PartialEq, Debug)]
pub struct TilemapBuilder {
    /// The type of grid to use.
    topology: GridTopology,
    /// An optional field which can contain the tilemaps dimensions in chunks.
    dimensions: Option<Dimension2>,
    /// The chunks dimensions in tiles.
    chunk_dimensions: Dimension3,
    /// The layer in the chunks offset value as X, Y. Each layer will be offset
    /// by this.
    layer_offset: Vec2,
    /// The tiles dimensions in pixels.
    texture_dimensions: Option<Dimension2>,
    /// The scale of a tile.
    tile_scale: Vec3,
    /// The amount of z layers.
    z_layers: usize,
    /// The layers to be set. If there are more, it will override `z_layers`.
    layers: Option<HashMap<usize, TilemapLayer>>,
    /// If the tilemap currently has a sprite sheet handle on it or not.
    texture_atlas: Option<Handle<TextureAtlas>>,
    /// Sets how many Z layers to render.
    render_depth: usize,
    /// True if this tilemap will automatically configure.
    auto_flags: AutoFlags,
    /// The radius of chunks to spawn from a camera's transform.
    auto_spawn: Option<Dimension2>,
}

impl Default for TilemapBuilder {
    fn default() -> Self {
        let layers = {
            let mut map = HashMap::default();
            map.insert(
                0,
                TilemapLayer {
                    kind: LayerKind::Dense,
                },
            );
            Some(map)
        };
        TilemapBuilder {
            topology: GridTopology::Square,
            dimensions: None,
            chunk_dimensions: DEFAULT_CHUNK_DIMENSIONS,
            layer_offset: Vec2::new(0., 0.),
            texture_dimensions: None,
            tile_scale: DEFAULT_TILE_SCALE.into(),
            z_layers: DEFAULT_Z_LAYERS,
            layers,
            texture_atlas: None,
            render_depth: 0,
            auto_flags: AutoFlags::NONE,
            auto_spawn: None,
        }
    }
}

impl TilemapBuilder {
    /// Configures the builder with the default settings.
    ///
    /// Is equivalent to [`default`] and [`builder`] method in the
    /// [tilemap]. Start with this then you are able to method chain.
    ///
    /// [`default`]: TilemapBuilder::default
    /// [`builder`]: TilemapBuilder
    /// [tilemap]: Tilemap
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    /// use bevy_tilemap::tilemap;
    ///
    /// let builder = TilemapBuilder::new();
    ///
    /// // Equivalent to...
    ///
    /// let builder = TilemapBuilder::default();
    ///
    /// // Or...
    ///
    /// let builder = Tilemap::builder();
    /// ```
    pub fn new() -> TilemapBuilder {
        TilemapBuilder::default()
    }

    /// Sets the topology of the tilemap.
    ///
    /// The default is a square grid. Use this if you want a hexagonal grid instead.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new().topology(GridTopology::HexY);
    /// ```
    pub fn topology(mut self, topology: GridTopology) -> TilemapBuilder {
        self.topology = topology;
        self
    }

    /// Sets the dimensions of the tilemap.
    ///
    /// If this is not set then the tilemap will be boundless entirely.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new().dimensions(5, 5);
    /// ```
    pub fn dimensions(mut self, width: u32, height: u32) -> TilemapBuilder {
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
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new().chunk_dimensions(32, 32, 1);
    /// ```
    pub fn chunk_dimensions(mut self, width: u32, height: u32, depth: u32) -> TilemapBuilder {
        self.chunk_dimensions = Dimension3::new(width, height, depth);
        self
    }

    /// Sets the layer offset as X, Y.
    ///
    /// Per layer, this is the offset that will take place. This is generally
    /// useful for 3D style tilemaps that need to give the illusion of depth.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    /// use bevy_math::Vec2;
    ///
    /// let builder = TilemapBuilder::new().layer_offset(Vec2::new(0.5, 0.5));
    /// ```
    pub fn layer_offset(mut self, offset: Vec2) -> TilemapBuilder {
        self.layer_offset = offset;
        self
    }

    /// Sets the tile dimensions.
    ///
    /// Tile dimensions are in pixels. If this is not set then the default of
    /// 32px, 32px is used.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new().texture_dimensions(32, 32);
    /// ```
    pub fn texture_dimensions(mut self, width: u32, height: u32) -> TilemapBuilder {
        self.texture_dimensions = Some(Dimension2::new(width, height));
        self
    }

    /// Sets the tile scale.
    ///
    /// By default this is the size of the texture for width and height which
    /// means that a tile width of 1 is the width in texture pixels for example.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new().tile_scale(32.0, 32.0, 32.0);
    /// ```
    pub fn tile_scale(mut self, width: f32, height: f32, depth: f32) -> TilemapBuilder {
        self.tile_scale = Vec3::new(width, height, depth);
        self
    }

    /// Sets the amount of render layers that sprites can exist on.
    ///
    /// By default there are 20 if this is not set.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new().z_layers(5);
    /// ```
    pub fn z_layers(mut self, layers: usize) -> TilemapBuilder {
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
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new()
    ///     .add_layer(TilemapLayer { kind: LayerKind::Dense, ..Default::default() }, 0)
    ///     .add_layer(TilemapLayer { kind: LayerKind::Sparse, ..Default::default() }, 1)
    ///     .add_layer(TilemapLayer { kind: LayerKind::Sparse, ..Default::default() }, 2);
    /// ```
    ///
    /// [`LayerKind`]: crate::chunk::LayerKind
    pub fn add_layer(mut self, layer: TilemapLayer, sprite_order: usize) -> TilemapBuilder {
        if let Some(layers) = &mut self.layers {
            layers.insert(sprite_order, layer);
        } else {
            let mut layers = HashMap::default();
            layers.insert(sprite_order, layer);
            self.layers = Some(layers);
        }
        self
    }

    /// Sets the texture atlas, this is **required** to be set.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let builder = TilemapBuilder::new().texture_atlas(texture_atlas_handle);
    /// ```
    pub fn texture_atlas(mut self, handle: Handle<TextureAtlas>) -> TilemapBuilder {
        self.texture_atlas = Some(handle);
        self
    }

    /// Sets if you want the tilemap to automatically spawn new chunks.
    ///
    /// This is useful if the tilemap map is meant to be endless or nearly
    /// endless with a defined size. Otherwise, it probably is better to spawn
    /// chunks directly or creating a system that can automatically spawn and
    /// despawn them given context.
    ///
    /// By default this is not enabled.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new().auto_chunk();
    /// ```
    pub fn auto_chunk(mut self) -> Self {
        self.auto_flags.toggle(AutoFlags::AUTO_CHUNK);
        self
    }

    /// Sets the tilemap to automatically spawn new chunks within given
    /// dimensions.
    ///
    /// This enables a feature which spawns just the right amount of chunks to
    /// fit the screen. It is possible that it may not be able to catch all
    /// dimensions but typical uses should be completely fine.
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new().auto_spawn(2, 3);
    /// ```
    pub fn auto_spawn(mut self, width: u32, height: u32) -> Self {
        self.auto_spawn = Some(Dimension2::new(width, height));
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
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let builder = TilemapBuilder::new().texture_dimensions(32, 32).texture_atlas(texture_atlas_handle);
    ///
    /// assert!(builder.finish().is_ok());
    /// assert!(TilemapBuilder::new().finish().is_err());
    /// ```
    ///
    /// [`texture_atlas`]: TilemapBuilder::texture_atlas
    /// [tilemap]: Tilemap
    /// [`TilemapError`]: TilemapError
    /// [`TilemapResult`]: TilemapResult
    pub fn finish(self) -> TilemapResult<Tilemap> {
        let texture_atlas = if let Some(atlas) = self.texture_atlas {
            atlas
        } else {
            return Err(ErrorKind::MissingTextureAtlas.into());
        };
        let texture_dimensions = if let Some(dimensions) = self.texture_dimensions {
            dimensions
        } else {
            return Err(ErrorKind::MissingTextureDimensions.into());
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

        let layer_count = if let Some(layers) = &self.layers {
            layers.iter().count()
        } else {
            0
        };
        let chunk_mesh =
            ChunkMesh::new(self.chunk_dimensions, layer_count as u32, self.layer_offset);

        let layers = {
            let mut layers = vec![None; z_layers];
            if let Some(map_layers) = self.layers {
                for (index, layer) in map_layers {
                    if let Some(l) = layers.get_mut(index) {
                        *l = Some(layer)
                    }
                }
            }
            layers
        };

        Ok(Tilemap {
            topology: self.topology,
            dimensions: self.dimensions,
            chunk_dimensions: self.chunk_dimensions,
            layer_offset: self.layer_offset,
            chunk_mesh,
            texture_dimensions,
            layers,
            auto_flags: self.auto_flags,
            auto_spawn: self.auto_spawn,
            custom_flags: Vec::new(),
            texture_atlas,
            chunks: Default::default(),
            entities: Default::default(),
            chunk_events: Default::default(),
            spawned: Default::default(),
        })
    }
}

impl TypeUuid for Tilemap {
    const TYPE_UUID: Uuid = Uuid::from_u128(109481186966523254410691740507722642628);
}

impl Default for Tilemap {
    fn default() -> Self {
        Tilemap {
            topology: GridTopology::Square,
            dimensions: None,
            chunk_dimensions: DEFAULT_CHUNK_DIMENSIONS,
            layer_offset: Vec2::default(),
            chunk_mesh: ChunkMesh::default(),
            texture_dimensions: DEFAULT_TEXTURE_DIMENSIONS,
            layers: vec![
                Some(TilemapLayer {
                    kind: LayerKind::Sparse,
                }),
                None,
                None,
                None,
                None,
            ],
            auto_flags: AutoFlags::NONE,
            auto_spawn: None,
            custom_flags: Vec::new(),
            texture_atlas: Handle::default(),
            chunks: Default::default(),
            entities: Default::default(),
            chunk_events: Default::default(),
            spawned: Default::default(),
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
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    /// ```
    ///
    /// [`default`]: Tilemap::default
    pub fn new(texture_atlas: Handle<TextureAtlas>, tile_width: u32, tile_height: u32) -> Tilemap {
        Tilemap {
            texture_atlas,
            texture_dimensions: Dimension2::new(tile_width, tile_height),
            ..Default::default()
        }
    }

    /// Configures the builder with the default settings.
    ///
    /// Is equivalent to [`default`] and [`builder`] method in the
    /// [tilemap]. Start with this then you are able to method chain.
    ///
    /// [`default`]: TilemapBuilder::default
    /// [`builder`]: Tilemap::builder
    /// [tilemap]: Tilemap
    ///
    /// # Examples
    /// ```
    /// use bevy_tilemap::prelude::*;
    ///
    /// let builder = TilemapBuilder::new();
    ///
    /// // Equivalent to...
    ///
    /// let builder = TilemapBuilder::default();
    ///
    /// // Or...
    ///
    /// let builder = Tilemap::builder();
    /// ```
    pub fn builder() -> TilemapBuilder {
        TilemapBuilder::default()
    }

    /// Sets the sprite sheet for use in the tilemap.
    ///
    /// This can be used if the need to swap the sprite sheet for another is
    /// wanted.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// let mut tilemap = Tilemap::default();
    ///
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
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    /// let texture_atlas: &Handle<TextureAtlas> = tilemap.texture_atlas();
    /// ```
    pub fn texture_atlas(&self) -> &Handle<TextureAtlas> {
        &self.texture_atlas
    }

    /// Constructs a new chunk and stores it at a coordinate position.
    ///
    /// It requires that you give it either a point. It then automatically sets
    /// both a sized mesh and chunk for use based on the parameters set in the
    /// parent tilemap.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .dimensions(3, 3)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// // Add some chunks.
    /// assert!(tilemap.insert_chunk((0, 0)).is_ok());
    /// assert!(tilemap.insert_chunk((1, 1)).is_ok());
    /// assert!(tilemap.insert_chunk((-2, -2)).is_err());
    ///
    /// assert!(tilemap.contains_chunk((0, 0)));
    /// assert!(tilemap.contains_chunk((1, 1)));
    /// assert!(!tilemap.contains_chunk((-2, -2)));
    /// ```
    /// # Errors
    ///
    /// If the point does not exist in the tilemap, an error is returned. This
    /// can only be returned if you had set the dimensions on the tilemap.
    ///
    /// Also will return an error if the chunk already exists. If this happens
    /// and was intentional, it is best to remove the chunk first. This is
    /// simply a fail safe without actually returning the chunk as it is meant
    /// to be kept internal.
    pub fn insert_chunk<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point: Point2 = point.into();
        if let Some(dimensions) = &self.dimensions {
            dimensions.check_point(point)?;
        }
        let layer_kinds = self
            .layers
            .iter()
            .map(|x| x.and_then(|y| Some(y.kind)))
            .collect::<Vec<Option<LayerKind>>>();
        let chunk = Chunk::new(point, &layer_kinds, self.chunk_dimensions);
        match self.chunks.insert(point, chunk) {
            Some(_) => Err(ErrorKind::ChunkAlreadyExists(point).into()),
            None => Ok(()),
        }
    }

    /// Returns `true` if the chunk is included in the tilemap.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// assert!(tilemap.insert_chunk((0, 0)).is_ok());
    /// assert!(tilemap.contains_chunk((0, 0)));
    /// assert!(!tilemap.contains_chunk((1, 1)));
    /// ```
    pub fn contains_chunk<P: Into<Point2>>(&mut self, point: P) -> bool {
        let point: Point2 = point.into();
        self.chunks.contains_key(&point)
    }

    #[deprecated(
        since = "0.4.0",
        note = "Please use `add_layer` method instead with the `TilemapLayer` struct"
    )]
    #[doc(hidden)]
    pub fn add_layer_with_kind(
        &mut self,
        kind: LayerKind,
        sprite_order: usize,
    ) -> TilemapResult<()> {
        let layer = TilemapLayer { kind };
        if let Some(some_kind) = self.layers.get_mut(sprite_order) {
            if some_kind.is_some() {
                return Err(ErrorKind::LayerExists(sprite_order).into());
            }
            *some_kind = Some(layer);
        }

        for chunk in self.chunks.values_mut() {
            chunk.add_sprite_layer(&kind, sprite_order, self.chunk_dimensions);
        }

        Ok(())
    }

    /// Adds a layer to the tilemap.
    ///
    /// ***Warning:*** This is very unwise and costly if there are many chunks
    /// in the tilemap. You should only add layers when creating the tilemap.
    /// The meshes for every single chunk has to be recalculated!
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
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let layer = TilemapLayer {
    ///    kind: LayerKind::Sparse,
    ///    ..Default::default()
    /// };
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// assert!(tilemap.add_layer(layer, 2).is_ok());
    /// assert!(tilemap.add_layer(layer, 2).is_err());
    /// ```
    ///
    /// [`add_layer_with_kind`]: Tilemap::add_layer_with_kind
    /// [`LayerKind`]: crate::chunk::LayerKind
    /// [`LayerKind::Sparse`]: crate::chunk::LayerKind::Sparse
    pub fn add_layer(&mut self, layer: TilemapLayer, sprite_layer: usize) -> TilemapResult<()> {
        if let Some(inner_layer) = self.layers.get_mut(sprite_layer) {
            if inner_layer.is_some() {
                return Err(ErrorKind::LayerExists(sprite_layer).into());
            }
            *inner_layer = Some(layer);
        }

        let mut layers = 0;
        for layer in self.layers().iter() {
            if layer.is_some() {
                layers += 1;
            }
        }
        let chunk_mesh = ChunkMesh::new(self.chunk_dimensions, layers, self.layer_offset);
        self.chunk_mesh = chunk_mesh;

        self.chunk_events.send(TilemapChunkEvent::AddLayer {
            layer_kind: layer.kind,
            sprite_layer,
        });

        Ok(())
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
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .z_layers(3)
    ///     .texture_dimensions(32, 32)
    ///     .add_layer(TilemapLayer { kind: LayerKind::Dense, ..Default::default() }, 0)
    ///     .add_layer(TilemapLayer { kind: LayerKind::Sparse, ..Default::default() }, 3)
    ///     .finish()
    ///     .unwrap();
    ///
    /// // If we moved this to layer 3, it would instead fail.
    /// assert!(tilemap.move_layer(0, 2).is_ok());
    /// assert!(tilemap.move_layer(3, 2).is_err());
    /// ```
    pub fn move_layer(
        &mut self,
        from_sprite_order: usize,
        to_sprite_order: usize,
    ) -> TilemapResult<()> {
        if let Some(layer) = self.layers.get(to_sprite_order) {
            if layer.is_some() {
                return Err(ErrorKind::LayerExists(to_sprite_order).into());
            }
        };
        if let Some(layer) = self.layers.get(from_sprite_order) {
            if Some(layer).is_none() {
                return Err(ErrorKind::LayerDoesNotExist(from_sprite_order).into());
            }
        }

        self.layers.swap(from_sprite_order, to_sprite_order);
        for chunk in self.chunks.values_mut() {
            chunk.move_sprite_layer(from_sprite_order, to_sprite_order);
        }

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
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// tilemap.add_layer(TilemapLayer { kind: LayerKind::Sparse, ..Default::default() }, 1);
    ///
    /// tilemap.remove_layer(1);
    /// ```
    ///
    /// [`move_layer`]: Tilemap::move_layer
    pub fn remove_layer(&mut self, z: usize) {
        if let Some(layer) = self.layers.get_mut(z) {
            *layer = None;
        } else {
            return;
        }

        for chunk in self.chunks.values_mut() {
            chunk.remove_sprite_layer(z);
        }
    }

    /// Spawns a chunk at a given index or coordinate.
    ///
    /// Does nothing if the chunk does not exist.
    ///
    /// # Errors
    ///
    /// If the coordinate or index is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .dimensions(1, 1)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// tilemap.insert_chunk((0, 0));
    ///
    /// // Ideally you should want to set some tiles here else nothing will
    /// // display in the render...
    ///
    /// assert!(tilemap.spawn_chunk((0, 0)).is_ok());
    /// assert!(tilemap.spawn_chunk((1, 1)).is_err());
    /// assert!(tilemap.spawn_chunk((-1, -1)).is_err());
    /// ```
    pub fn spawn_chunk<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point: Point2 = point.into();
        if let Some(dimensions) = &self.dimensions {
            dimensions.check_point(point)?;
        }

        if self.spawned.contains(&(point.x, point.y)) {
            return Ok(());
        } else {
            self.chunk_events.send(TilemapChunkEvent::Spawned { point });
        }

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
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .chunk_dimensions(32, 32, 1)
    ///     .texture_dimensions(32, 32)
    ///     .dimensions(1, 1)
    ///     .finish()
    ///     .unwrap();
    ///
    /// let point = (15, 15);
    /// let sprite_index = 0;
    /// let tile = Tile { point, sprite_index, ..Default::default() };
    ///
    /// tilemap.insert_tile(tile);
    ///
    /// assert!(tilemap.spawn_chunk_containing_point(point).is_ok());
    /// assert!(tilemap.spawn_chunk_containing_point((16, 16)).is_err());
    /// assert!(tilemap.spawn_chunk_containing_point((-18, -18)).is_err());
    /// ```
    pub fn spawn_chunk_containing_point<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point = self.point_to_chunk_point(point);
        self.spawn_chunk(point)
    }

    /// De-spawns a spawned chunk at a given index or coordinate.
    ///
    /// If the chunk is not spawned this will result in nothing.
    ///
    /// # Errors
    ///
    /// If the coordinate or index is out of bounds, an error will be returned.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .dimensions(1, 1)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// assert!(tilemap.insert_chunk((0, 0)).is_ok());
    ///
    /// // Ideally you should want to set some tiles here else nothing will
    /// // display in the render...
    ///
    /// assert!(tilemap.spawn_chunk((0, 0)).is_ok());
    ///
    /// // Later a frame or more on...
    ///
    /// assert!(tilemap.despawn_chunk((0, 0)).is_ok());
    /// assert!(tilemap.despawn_chunk((-1, -1)).is_err());
    /// ```
    pub fn despawn_chunk<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point: Point2 = point.into();
        if let Some(dimensions) = &self.dimensions {
            dimensions.check_point(point)?;
        }

        self.spawned.remove(&(point.x, point.y));

        if self.chunks.get_mut(&point).is_some() {
            self.chunk_events
                .send(TilemapChunkEvent::Despawned { point });
            Ok(())
        } else {
            Err(ErrorKind::MissingChunk.into())
        }
    }

    /// Destructively removes a chunk at a coordinate position and despawns them
    /// if needed.
    ///
    /// Internally, this sends an event to the tilemap's system flagging which
    /// chunks must be removed by index and entity. A chunk is not recoverable
    /// if this action is done.
    ///
    /// Does nothing if the chunk does not exist.
    ///
    /// # Errors
    ///
    /// If the coordinate or index is out of bounds, an error will be returned.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .dimensions(3, 3)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// // Add some chunks.
    /// assert!(tilemap.insert_chunk((0, 0)).is_ok());
    /// assert!(tilemap.insert_chunk((1, 1)).is_ok());
    ///
    /// assert!(tilemap.remove_chunk((0, 0)).is_ok());
    /// assert!(tilemap.remove_chunk((1, 1)).is_ok());
    /// assert!(tilemap.remove_chunk((-2, -2)).is_err());
    /// ```
    pub fn remove_chunk<P: Into<Point2>>(&mut self, point: P) -> TilemapResult<()> {
        let point = point.into();
        self.despawn_chunk(point)?;

        self.chunks.remove(&point);

        Ok(())
    }

    /// Takes a tile point and changes it into a chunk point.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// let tile_point = (15, 15);
    /// let chunk_point = tilemap.point_to_chunk_point(tile_point);
    /// assert_eq!((0, 0), chunk_point);
    ///
    /// let tile_point = (16, 16);
    /// let chunk_point = tilemap.point_to_chunk_point(tile_point);
    /// assert_eq!((1, 1), chunk_point);
    ///
    /// let tile_point = (-16, -16);
    /// let chunk_point = tilemap.point_to_chunk_point(tile_point);
    /// assert_eq!((-0, -0), chunk_point);
    ///
    /// let tile_point = (-17, -17);
    /// let chunk_point = tilemap.point_to_chunk_point(tile_point);
    /// assert_eq!((-1, -1), chunk_point);
    /// ```
    pub fn point_to_chunk_point<P: Into<Point2>>(&self, point: P) -> (i32, i32) {
        let point: Point2 = point.into();
        let width = self.chunk_dimensions.width as f32;
        let height = self.chunk_dimensions.height as f32;
        let x = ((point.x as f32 + width / 2.0) / width).floor() as i32;
        let y = ((point.y as f32 + height / 2.0) / height).floor() as i32;
        (x, y)
    }

    /// Sorts tiles into the chunks they belong to.
    fn sort_tiles_to_chunks<P, I>(
        &mut self,
        tiles: I,
    ) -> TilemapResult<HashMap<Point2, Vec<Tile<Point3>>>>
    where
        P: Into<Point3>,
        I: IntoIterator<Item = Tile<P>>,
    {
        let width = self.chunk_dimensions.width as i32;
        let height = self.chunk_dimensions.height as i32;

        let mut chunk_map: HashMap<Point2, Vec<Tile<Point3>>> = HashMap::default();
        for tile in tiles.into_iter() {
            let global_tile_point: Point3 = tile.point.into();
            let chunk_point: Point2 = self.point_to_chunk_point(global_tile_point).into();

            if let Some(layer) = self.layers.get(tile.sprite_order as usize) {
                if layer.as_ref().is_none() {
                    self.add_layer(TilemapLayer::default(), tile.sprite_order as usize)?;
                }
            } else {
                return Err(ErrorKind::LayerDoesNotExist(tile.sprite_order).into());
            }

            let tile_point = Point3::new(
                global_tile_point.x - (width * chunk_point.x) + (width / 2),
                global_tile_point.y - (height * chunk_point.y) + (height / 2),
                global_tile_point.z,
            );

            let chunk_tile: Tile<Point3> = Tile {
                point: tile_point,
                sprite_order: tile.sprite_order,
                sprite_index: tile.sprite_index,
                tint: tile.tint,
            };
            if let Some(tiles) = chunk_map.get_mut(&chunk_point) {
                tiles.push(chunk_tile);
            } else {
                let tiles = vec![chunk_tile];
                chunk_map.insert(chunk_point, tiles);
            }
        }
        Ok(chunk_map)
    }

    /// Sets many tiles, creating new chunks if needed.
    ///
    /// If setting a single tile is more preferable, then use the [`insert_tile`]
    /// method instead.
    ///
    /// If the chunk does not yet exist, it will create a new one automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if the given coordinate or index is out of bounds, the
    /// layer or chunk does not exist. If either the layer or chunk error occurs
    /// then creating what is missing will resolve it.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_render::prelude::*;
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::{prelude::*, chunk::RawTile};
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .dimensions(1, 1)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// tilemap.insert_chunk((0, 0)).unwrap();
    ///
    /// let mut tiles = vec![
    ///     Tile { point: (1, 1), sprite_index: 0, ..Default::default() },
    ///     Tile { point: (2, 2), sprite_index: 1, ..Default::default() },
    ///     Tile { point: (3, 3), sprite_index: 2, ..Default::default() },
    /// ];
    ///
    /// // Set multiple tiles and unwrap the result
    /// tilemap.insert_tiles(tiles).unwrap();
    ///
    /// assert_eq!(tilemap.get_tile((1, 1), 0), Some(&RawTile { index: 0, color: Color::WHITE }));
    /// assert_eq!(tilemap.get_tile((2, 2), 0), Some(&RawTile { index: 1, color: Color::WHITE }));
    /// assert_eq!(tilemap.get_tile((3, 3), 0), Some(&RawTile { index: 2, color: Color::WHITE }));
    /// assert_eq!(tilemap.get_tile((4, 4), 0), None);
    /// ```
    ///
    /// [`insert_tile`]: Tilemap::insert_tile
    pub fn insert_tiles<P, I>(&mut self, tiles: I) -> TilemapResult<()>
    where
        P: Into<Point3>,
        I: IntoIterator<Item = Tile<P>>,
    {
        let chunk_map = self.sort_tiles_to_chunks(tiles)?;
        for (chunk_point, tiles) in chunk_map.into_iter() {
            // Is there a better way to do this? Clippy hates if I don't do it
            // like this talking about constructing regardless yet, here it is,
            // copying stuff regardless because it doesn't like self in the
            // `FnOnce`.
            let layers = self.layers.clone();
            let chunk_dimensions = self.chunk_dimensions;
            let chunk = if self.auto_flags.contains(AutoFlags::AUTO_CHUNK) {
                self.chunks.entry(chunk_point).or_insert_with(|| {
                    let layer_kinds = layers
                        .iter()
                        .map(|x| x.and_then(|y| Some(y.kind)))
                        .collect::<Vec<Option<LayerKind>>>();
                    Chunk::new(chunk_point, &layer_kinds, chunk_dimensions)
                })
            } else {
                match self.chunks.get_mut(&chunk_point) {
                    Some(c) => c,
                    None => return Err(ErrorKind::MissingChunk.into()),
                }
            };

            for tile in tiles.iter() {
                let index = self.chunk_dimensions.encode_point_unchecked(tile.point);
                chunk.set_tile(index, *tile);
            }

            if chunk.mesh().is_some() {
                self.chunk_events.send(TilemapChunkEvent::Modified {
                    point: chunk.point(),
                });
            }
        }

        Ok(())
    }

    /// Sets a single tile at a coordinate position, creating a chunk if necessary.
    ///
    /// If you are setting more than one tile at a time, it is highly
    /// recommended not to run this method! If that is preferred, do use
    /// [`insert_tiles`] instead. Every single tile that is created creates a new
    /// event. With bulk tiles, it creates 1 event for all.
    ///
    /// If the chunk does not yet exist, it will create a new one automatically.
    ///
    /// [`insert_tiles`]: Tilemap::insert_tiles
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_render::prelude::*;
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::{prelude::*, chunk::RawTile};
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// tilemap.insert_chunk((0, 0)).unwrap();
    ///
    /// let point = (9, 3);
    /// let sprite_index = 3;
    /// let tile = Tile { point, sprite_index, ..Default::default() };
    ///
    /// assert!(tilemap.insert_tile(tile).is_ok());
    /// assert_eq!(tilemap.get_tile((9, 3), 0), Some(&RawTile { index: 3, color: Color::WHITE }))
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the given coordinate or index is out of bounds.
    pub fn insert_tile<P: Into<Point3>>(&mut self, tile: Tile<P>) -> TilemapResult<()> {
        let tiles = vec![tile];
        self.insert_tiles(tiles)
    }

    /// Clears the tiles at the specified points from the tilemap.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_render::prelude::*;
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::{prelude::*, chunk::RawTile};
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// assert!(tilemap.insert_chunk((0, 0)).is_ok());
    ///
    /// let mut tiles = vec![
    ///     Tile { point: (1, 1, 0), ..Default::default() },
    ///     Tile { point: (2, 2, 0), ..Default::default() },
    ///     Tile { point: (3, 3, 0), ..Default::default() },
    /// ];
    ///
    /// // Set multiple tiles and unwrap the result
    /// assert!(tilemap.insert_tiles(tiles.clone()).is_ok());
    ///
    /// // Then later on... Do note that if this done in the same frame, the
    /// // tiles will not even exist at all.
    /// let mut to_remove = vec![
    ///     ((1, 1), 0),
    ///     ((2, 2), 0),
    /// ];
    ///
    /// tilemap.clear_tiles(to_remove).unwrap();
    /// assert_eq!(tilemap.get_tile((1, 1, 0), 0), None);
    /// assert_eq!(tilemap.get_tile((2, 2, 0), 0), None);
    /// assert_eq!(tilemap.get_tile((3, 3, 0), 0), Some(&RawTile { index: 0, color: Color::WHITE} ));
    /// ```
    ///
    /// # Errors
    ///
    /// An error can occure if the point is outside of the tilemap. This can
    /// only happen if the tilemap has dimensions.
    pub fn clear_tiles<P, I>(&mut self, points: I) -> TilemapResult<()>
    where
        P: Into<Point3>,
        I: IntoIterator<Item = (P, usize)>,
    {
        let mut tiles = Vec::new();
        for (point, sprite_order) in points {
            tiles.push(Tile {
                point: point.into(),
                sprite_index: 0,
                sprite_order,
                tint: Color::rgba(0.0, 0.0, 0.0, 0.0),
            });
        }
        let chunk_map = self.sort_tiles_to_chunks(tiles)?;
        for (chunk_point, tiles) in chunk_map.into_iter() {
            let chunk = match self.chunks.get_mut(&chunk_point) {
                Some(c) => c,
                None => return Err(ErrorKind::MissingChunk.into()),
            };
            for tile in tiles.iter() {
                let index = self.chunk_dimensions.encode_point_unchecked(tile.point);
                chunk.remove_tile(index, tile.sprite_order, tile.point.z as usize);
            }

            self.chunk_events.send(TilemapChunkEvent::Modified {
                point: chunk.point(),
            });
        }

        Ok(())
    }

    /// Takes a global tile point and returns a tile point in a chunk.
    fn point_to_tile_point(&self, point: Point3) -> Point3 {
        let chunk_point: Point2 = self.point_to_chunk_point(point).into();
        let width = self.chunk_dimensions.width as i32;
        let height = self.chunk_dimensions.height as i32;
        Point3::new(
            point.x - (width * chunk_point.x) + (width / 2),
            point.y - (height * chunk_point.y) + (height / 2),
            point.z,
        )
    }

    /// Clear a single tile at the specified point from the tilemap.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::{prelude::*, chunk::RawTile};
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// assert!(tilemap.insert_chunk((0, 0)).is_ok());
    ///
    /// let point = (3, 1);
    /// let sprite_index = 1;
    /// let tile = Tile { point, sprite_index, ..Default::default() };
    ///
    /// // Set a single tile and unwrap the result
    /// assert!(tilemap.insert_tile(tile).is_ok());
    ///
    /// // Later on...
    /// assert!(tilemap.clear_tile(point, 0).is_ok());
    /// assert_eq!(tilemap.get_tile((3, 1), 0), None);
    /// ```
    ///
    /// # Errors
    ///
    /// An error can occure if the point is outside of the tilemap. This can
    /// only happen if the tilemap has dimensions.
    pub fn clear_tile<P>(&mut self, point: P, sprite_order: usize) -> TilemapResult<()>
    where
        P: Into<Point3>,
    {
        let points = vec![(point, sprite_order)];
        self.clear_tiles(points)
    }

    /// Gets a raw tile from a given point and z order.
    ///
    /// This is different thant he usual [`Tile`] struct in that it only
    /// contains the sprite index and the tint.
    ///
    /// [`Tile`]: crate::tile::Tile
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_render::prelude::*;
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::{prelude::*, chunk::RawTile};
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// tilemap.insert_chunk((0, 0)).unwrap();
    ///
    /// let point = (9, 3);
    /// let sprite_index = 3;
    /// let tile = Tile { point, sprite_index, ..Default::default() };
    ///
    /// assert!(tilemap.insert_tile(tile).is_ok());
    /// assert_eq!(tilemap.get_tile((9, 3), 0), Some(&RawTile { index: 3, color: Color::WHITE }));
    /// assert_eq!(tilemap.get_tile((10, 4), 0), None);
    /// ```
    pub fn get_tile<P>(&mut self, point: P, sprite_order: usize) -> Option<&RawTile>
    where
        P: Into<Point3>,
    {
        let point: Point3 = point.into();
        let chunk_point: Point2 = self.point_to_chunk_point(point).into();
        let tile_point = self.point_to_tile_point(point);
        let chunk = self.chunks.get(&chunk_point)?;
        let index = self.chunk_dimensions.encode_point_unchecked(tile_point);
        chunk.get_tile(index, sprite_order, point.z as usize)
    }

    /// Gets a mutable raw tile from a given point and z order.
    ///
    /// This is different thant he usual [`Tile`] struct in that it only
    /// contains the sprite index and the tint.
    ///
    /// [`Tile`]: crate::tile::Tile
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_render::prelude::*;
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::{prelude::*, chunk::RawTile};
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// tilemap.insert_chunk((0, 0)).unwrap();
    ///
    /// let point = (2, 5);
    /// let sprite_index = 2;
    /// let tile = Tile { point, sprite_index, ..Default::default() };
    ///
    /// assert!(tilemap.insert_tile(tile).is_ok());
    /// assert_eq!(tilemap.get_tile_mut((2, 5), 0), Some(&mut RawTile { index: 2, color: Color::WHITE }));
    /// assert_eq!(tilemap.get_tile_mut((1, 4), 0), None);
    /// ```
    pub fn get_tile_mut<P>(&mut self, point: P, sprite_order: usize) -> Option<&mut RawTile>
    where
        P: Into<Point3>,
    {
        let point: Point3 = point.into();
        let chunk_point: Point2 = self.point_to_chunk_point(point).into();
        let tile_point = self.point_to_tile_point(point);
        let chunk = self.chunks.get_mut(&chunk_point)?;
        let index = self.chunk_dimensions.encode_point_unchecked(tile_point);
        let mut layers = HashMap::default();
        layers.insert(sprite_order, chunk_point);
        self.chunk_events.send(TilemapChunkEvent::Modified {
            point: chunk.point(),
        });
        chunk.get_tile_mut(index, sprite_order, point.z as usize)
    }

    /// Returns the center tile, if the tilemap has dimensions.
    ///
    /// Returns `None` if the tilemap has no constrainted dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let mut tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle.clone_weak())
    ///     .dimensions(32, 32)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// let center = tilemap.center_tile_coord();
    ///
    /// // 32 * 32 / 2 = 512
    /// assert_eq!(center, Some((512, 512)));
    ///
    /// let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// let center = tilemap.center_tile_coord();
    ///
    /// assert_eq!(center, None);
    /// ```
    pub fn center_tile_coord(&self) -> Option<(i32, i32)> {
        self.dimensions.map(|dimensions| {
            (
                (dimensions.width / 2 * self.chunk_dimensions.width) as i32,
                (dimensions.height / 2 * self.chunk_dimensions.height) as i32,
            )
        })
    }

    /// The width of the tilemap in chunks, if it has dimensions.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle.clone_weak())
    ///     .dimensions(32, 64)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// let width = tilemap.width();
    ///
    /// assert_eq!(width, Some(32));
    ///
    /// let tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// let width = tilemap.width();
    ///
    /// assert_eq!(width, None);
    /// ```
    pub fn width(&self) -> Option<u32> {
        self.dimensions.map(|dimensions| dimensions.width)
    }

    /// The height of the tilemap in chunks, if it has dimensions.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle.clone_weak())
    ///     .dimensions(32, 64)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// let height = tilemap.height();
    ///
    /// assert_eq!(height, Some(64));
    ///
    /// let tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// let height = tilemap.height();
    ///
    /// assert_eq!(height, None);
    /// ```
    pub fn height(&self) -> Option<u32> {
        self.dimensions.map(|dimensions| dimensions.height)
    }

    /// The width of all the chunks in tiles.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .chunk_dimensions(32, 64, 1)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// let chunk_width: u32 = tilemap.chunk_width();
    ///
    /// assert_eq!(chunk_width, 32);
    /// ```
    pub fn chunk_width(&self) -> u32 {
        self.chunk_dimensions.width
    }

    /// The height of all the chunks in tiles.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .chunk_dimensions(32, 64, 1)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// let chunk_height: u32 = tilemap.chunk_height();
    ///
    /// assert_eq!(chunk_height, 64);
    /// ```
    pub fn chunk_height(&self) -> u32 {
        self.chunk_dimensions.height
    }

    /// The width of a tile in pixels.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .texture_dimensions(32, 64)
    ///     .finish()
    ///     .unwrap();
    ///
    /// let tile_width: u32 = tilemap.tile_width();
    ///
    /// assert_eq!(tile_width, 32);
    /// ```
    pub fn tile_width(&self) -> u32 {
        self.texture_dimensions.width
    }

    /// The height of a tile in pixels.
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .texture_dimensions(32, 64)
    ///     .finish()
    ///     .unwrap();
    ///
    /// let tile_height: u32 = tilemap.tile_height();
    ///
    /// assert_eq!(tile_height, 64);
    /// ```
    pub fn tile_height(&self) -> u32 {
        self.texture_dimensions.height
    }

    /// Gets a reference to a chunk.
    pub(crate) fn get_chunk(&self, point: &Point2) -> Option<&Chunk> {
        self.chunks.get(point)
    }

    /// The topology of the tilemap grid.
    ///
    /// Currently there are 7 topologies which are set with [`GridTopology`]. By
    /// default this is square as it is the most common topology.
    ///
    /// Typically, for most situations squares are used for local maps and hex
    /// is used for war games or world maps. It is easier to define structures
    /// with walls and floors with square but not impossible with hex.
    ///
    /// [`GridTopology`]: crate::render::GridTopology
    ///
    /// # Examples
    /// ```
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::prelude::*;
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = TilemapBuilder::new()
    ///     .texture_atlas(texture_atlas_handle)
    ///     .topology(GridTopology::HexX)
    ///     .texture_dimensions(32, 32)
    ///     .finish()
    ///     .unwrap();
    ///
    /// assert_eq!(tilemap.topology(), GridTopology::HexX);
    /// ```
    pub fn topology(&self) -> GridTopology {
        self.topology
    }

    /// Returns a reference to the tilemap chunk events.
    ///
    /// This is handy if it is needed to know when new chunks are created which
    /// can then be used to trigger events with other systems. For example,
    /// if you have a system that adds tiles procedurally to the chunks, upon
    /// a chunk event this can be used to trigger the creation of those tiles.
    ///
    /// # Examples
    /// ```
    /// use bevy_app::prelude::*;
    /// use bevy_asset::{prelude::*, HandleId};
    /// use bevy_sprite::prelude::*;
    /// use bevy_tilemap::{prelude::*, event::TilemapChunkEvent};
    ///
    /// // In production use a strong handle from an actual source.
    /// let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    ///
    /// let tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
    ///
    /// let events: &Events<TilemapChunkEvent> = tilemap.chunk_events();
    /// ```
    pub fn chunk_events(&self) -> &Events<TilemapChunkEvent> {
        &self.chunk_events
    }

    /// Updates the chunk events. This should only be done once per frame.
    pub(crate) fn chunk_events_update(&mut self) {
        self.chunk_events.update()
    }

    /// Returns an option containing a Dimension2.
    pub(crate) fn auto_spawn(&self) -> Option<Dimension2> {
        self.auto_spawn
    }

    /// Sets the auto spawn radius.
    pub(crate) fn set_auto_spawn(&mut self, dimension: Dimension2) {
        self.auto_spawn = Some(dimension);
    }

    /// Returns a copy of the chunk's dimensions.
    pub(crate) fn chunk_dimensions(&self) -> Dimension3 {
        self.chunk_dimensions
    }

    /// Returns a copy of the chunk's texture dimensions.
    pub(crate) fn texture_dimensions(&self) -> Dimension2 {
        self.texture_dimensions
    }

    /// Returns a reference to the hash set of spawned chunks.
    pub(crate) fn spawned_chunks(&self) -> &HashSet<(i32, i32)> {
        &self.spawned
    }

    /// Returns a mutable reference to the spawned chunk points.
    pub(crate) fn spawned_chunks_mut(&mut self) -> &mut HashSet<(i32, i32)> {
        &mut self.spawned
    }

    /// Returns a reference to the layers in the tilemap.
    pub(crate) fn layers(&self) -> Vec<Option<TilemapLayer>> {
        self.layers.clone()
    }

    /// Returns a reference to the chunks in the tilemap.
    pub(crate) fn chunks(&self) -> &HashMap<Point2, Chunk> {
        &self.chunks
    }

    /// Returns a mutable reference to the inner chunks.
    pub(crate) fn chunks_mut(&mut self) -> &mut HashMap<Point2, Chunk> {
        &mut self.chunks
    }

    /// A reference of a chunk's mesh.
    pub(crate) fn chunk_mesh(&self) -> &ChunkMesh {
        &self.chunk_mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Tilemap {
        /// Flags a tilemap chunk that it has been modified. Intended for testing
        /// purposes only.
        pub(crate) fn modify_chunk(&mut self, point: Point2) {
            self.chunk_events
                .send(TilemapChunkEvent::Modified { point });
        }
    }

    // fn new_tilemap_no_auto() -> Tilemap {
    //     let texture_atlas_handle = Handle::weak(Handllet modified_layer = layer_query.get()eId::random::<TextureAtlas>());

    //     let mut tilemap = Tilemap::builder()
    //         .chunk_dimensions(5, 5)
    //         .texture_atlas(texture_atlas_handle)
    //         .finish()
    //         .unwrap();

    //     tilemap
    // }

    // #[test]
    // fn insert_chunks() {
    //     let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
    //     let mut tilemap = Tilemap::new(texture_atlas_handle);

    //     tilemap.insert_chunk(Point2::new(0, 0)).unwrap();
    //     tilemap.insert_chunk(Point2::new(1, -1)).unwrap();
    //     tilemap.insert_chunk(Point2::new(1, 1)).unwrap();
    //     tilemap.insert_chunk(Point2::new(-1, -1)).unwrap();
    // }
}
