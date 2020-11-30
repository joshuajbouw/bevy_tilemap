//! Tiles organised into chunks for efficiency and performance.
//!
//! Mostly everything in this module is private API and not intended to be used
//! outside of this crate as a lot goes on under the hood that can cause issues.
//! With that being said, everything that can be used with helping a chunk get
//! created does live in here.
//!
//! These below examples have nothing to do with this library as all should be
//! done through the [`Tilemap`]. These are just more specific examples which
//! use the private API of this library.
//!
//! [`Tilemap`]: crate::tilemap::Tilemap
//!
//! # Simple chunk creation
//! ```
//! use bevy_tilemap::prelude::*;
//! use bevy::asset::HandleId;
//! use bevy::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::new(texture_atlas_handle);
//!
//! // There are two ways to create a new chunk. Either directly...
//!
//! tilemap.new_chunk((0, 0));
//!
//! // Or indirectly...
//!
//! let point = (0, 0);
//! let sprite_index = 0;
//! let tile = Tile::new(point.clone(), sprite_index);
//! tilemap.insert_tile(tile);
//!
//! ```
//!
//! # Specifying what kind of chunk
//! ```
//! use bevy_tilemap::prelude::*;
//! use bevy::asset::HandleId;
//! use bevy::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::new(texture_atlas_handle);
//!
//! tilemap.new_chunk((0, 0));
//!
//! let z_order = 0;
//! tilemap.add_layer_with_kind(LayerKind::Dense, 0);
//!
//! let z_order = 1;
//! tilemap.add_layer_with_kind(LayerKind::Sparse, 1);
//! ```
use crate::{
    entity::{ChunkDimensions, DirtyLayer},
    lib::*,
    mesh::ChunkMesh,
    tile::RawTile,
};

/// Common methods for layers in a chunk.
pub(crate) trait Layer: 'static {
    /// Returns the handle of the mesh.
    fn mesh(&self) -> &Handle<Mesh>;

    /// Sets the mesh for the layer.
    fn set_mesh(&mut self, mesh: Handle<Mesh>);

    /// Sets a raw tile for a layer at an index.
    fn set_raw_tile(&mut self, index: usize, tile: RawTile);

    /// Takes all the tiles in the layer and returns attributes for the renderer.
    fn tiles_to_attributes(&self, area: usize) -> (Vec<f32>, Vec<[f32; 4]>);
}

/// A layer with dense sprite tiles.
///
/// The difference between a dense layer and a sparse layer is simply the
/// storage types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DenseLayer {
    /// A mesh handle.
    #[cfg_attr(feature = "serde", serde(skip))]
    mesh: Handle<Mesh>,
    /// A vector of all the tiles in the chunk.
    tiles: Vec<RawTile>,
}

impl Layer for DenseLayer {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    fn set_mesh(&mut self, mesh: Handle<Mesh>) {
        self.mesh = mesh;
    }

    fn set_raw_tile(&mut self, index: usize, tile: RawTile) {
        if let Some(inner_tile) = self.tiles.get_mut(index) {
            *inner_tile = tile;
        } // TODO: Else statement with an ERR log when released
    }

    fn tiles_to_attributes(&self, _area: usize) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::dense_tiles_to_attributes(&self.tiles)
    }
}

impl DenseLayer {
    /// Constructs a new dense layer with tiles.
    pub(crate) fn new(tiles: Vec<RawTile>) -> DenseLayer {
        DenseLayer {
            mesh: Default::default(),
            tiles,
        }
    }
}

/// A layer with sparse sprite tiles.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct SparseLayer {
    /// A mesh handle.
    #[cfg_attr(feature = "serde", serde(skip))]
    mesh: Handle<Mesh>,
    /// A map of all the tiles in the chunk.
    tiles: HashMap<usize, RawTile>,
}

impl Layer for SparseLayer {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    fn set_mesh(&mut self, mesh: Handle<Mesh>) {
        self.mesh = mesh;
    }

    fn set_raw_tile(&mut self, index: usize, tile: RawTile) {
        if tile.color.a() == 0.0 {
            self.tiles.remove(&index);
        }
        self.tiles.insert(index, tile);
    }

    fn tiles_to_attributes(&self, area: usize) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::sparse_tiles_to_attributes(area, &self.tiles)
    }
}

impl SparseLayer {
    /// Constructs a new sparse layer with a tile hashmap.
    pub(crate) fn new(tiles: HashMap<usize, RawTile>) -> SparseLayer {
        SparseLayer {
            mesh: Default::default(),
            tiles,
        }
    }
}

/// Specifies which kind of layer to construct, either a dense or a sparse
/// sprite layer.
///
/// The difference between a dense and sparse layer is namely the storage kind.
/// A dense layer uses a vector and must fully contain tiles. This is ideal for
/// backgrounds. A sparse layer on the other hand uses a map with coordinates
/// to a tile. This is ideal for entities, objects or items.
///
/// It is highly recommended to adhere to the above principles to get the lowest
/// amount of byte usage.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum LayerKind {
    /// Specifies the tilemap to add a dense sprite layer.
    Dense,
    /// Specifies the tilemap to add a sparse sprite layer.
    Sparse,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
/// Inner enum used for storing either a dense or sparse layer.
enum LayerKindInner {
    /// Inner dense layer storage.
    Dense(DenseLayer),
    /// Inner sparse layer storage.
    Sparse(SparseLayer),
}

impl AsRef<dyn Layer> for LayerKindInner {
    fn as_ref(&self) -> &dyn Layer {
        match self {
            LayerKindInner::Dense(s) => s,
            LayerKindInner::Sparse(s) => s,
        }
    }
}

impl AsMut<dyn Layer> for LayerKindInner {
    fn as_mut(&mut self) -> &mut dyn Layer {
        match self {
            LayerKindInner::Dense(s) => s,
            LayerKindInner::Sparse(s) => s,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
/// A sprite layer which can either store a sparse or dense layer.
pub(crate) struct SpriteLayer {
    /// Enum storage of the kind of layer.
    inner: LayerKindInner,
    #[cfg_attr(feature = "serde", serde(skip))]
    /// Contains an entity if the layer had been spawned.
    entity: Option<Entity>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug, TypeUuid)]
#[uuid = "47691827-0b89-4474-a14e-f2ea3c88320f"]
#[doc(hidden)]
pub struct Chunk {
    point: Point2,
    sprite_layers: Vec<Option<SpriteLayer>>,
}

impl Chunk {
    /// A newly constructed chunk from a point and the maximum number of layers.
    pub(crate) fn new(point: Point2, max_layers: usize) -> Chunk {
        Chunk {
            point,
            sprite_layers: vec![None; max_layers],
        }
    }

    /// Adds a layer from a layer kind, the z layer, and dimensions of the
    /// chunk.
    pub(crate) fn add_layer(&mut self, kind: &LayerKind, z: usize, dimensions: Dimension2) {
        match kind {
            LayerKind::Dense => {
                let tiles = vec![
                    RawTile {
                        index: 0,
                        color: Color::rgba(0.0, 0.0, 0.0, 0.0)
                    };
                    dimensions.area() as usize
                ];
                if let Some(layer) = self.sprite_layers.get_mut(z) {
                    *layer = Some(SpriteLayer {
                        inner: LayerKindInner::Dense(DenseLayer::new(tiles)),
                        entity: None,
                    });
                } // TODO: Else statement with an ERR log when released
            }
            LayerKind::Sparse => {
                if let Some(layer) = self.sprite_layers.get_mut(z) {
                    *layer = Some(SpriteLayer {
                        inner: LayerKindInner::Sparse(SparseLayer::new(HashMap::default())),
                        entity: None,
                    });
                } // TODO: Else statement with an ERR log when released
            }
        }
    }

    /// Returns the point of the location of the chunk.
    pub(crate) fn point(&self) -> Point2 {
        self.point
    }

    /// Moves a layer from a z layer to another.
    pub(crate) fn move_layer(&mut self, from_z: usize, to_z: usize) {
        // TODO: rename to swap and include it in the greater api
        if self.sprite_layers.get(to_z).is_some() {
            panic!("Sprite layer {} unexpectedly exists.", to_z);
        }

        self.sprite_layers.swap(from_z, to_z);
    }

    /// Removes a layer from the specified layer.
    pub(crate) fn remove_layer(&mut self, z_order: usize) {
        self.sprite_layers.get_mut(z_order).take();
    }

    /// Sets the mesh for the chunk layer to use.
    pub(crate) fn set_mesh(&mut self, z_order: usize, mesh: Handle<Mesh>) {
        if let Some(layer) = self.sprite_layers.get_mut(z_order) {
            if let Some(layer) = layer.as_mut() {
                layer.inner.as_mut().set_mesh(mesh)
            } // TODO: Bevy log error when implemented
        } // TODO: Bevy log error when implemented
    }

    /// Sets a single raw tile to be added to a z layer and index.
    pub(crate) fn set_raw_tile(&mut self, z_order: usize, index: usize, raw_tile: RawTile) {
        if let Some(layer) = self.sprite_layers.get_mut(z_order) {
            if let Some(layer) = layer.as_mut() {
                layer.inner.as_mut().set_raw_tile(index, raw_tile);
            } // TODO: Bevy log error when implemented
        } // TODO: Bevy log error when implemented
    }

    /// Adds an entity to a z layer, always when it is spawned.
    pub(crate) fn add_entity(&mut self, z_order: usize, entity: Entity) {
        if let Some(layer) = self.sprite_layers.get_mut(z_order) {
            if let Some(layer) = layer.as_mut() {
                layer.entity = Some(entity);
            } // TODO: Bevy log error when implemented
        } // TODO: Bevy log error when implemented
    }

    /// Gets the layers entity, if any. Useful for despawning.
    pub(crate) fn get_entity(&self, z_order: usize) -> Option<Entity> {
        self.sprite_layers
            .get(z_order)
            .and_then(|o| o.as_ref().and_then(|layer| layer.entity))
    }

    /// Gets all the layers entities for use with bulk despawning.
    pub(crate) fn get_entities(&self) -> Vec<Entity> {
        let mut entities = Vec::new();
        for sprite_layer in &self.sprite_layers {
            if let Some(layer) = sprite_layer {
                if let Some(entity) = layer.entity {
                    entities.push(entity);
                }
            }
        }
        entities
    }

    /// At the given z layer, changes the tiles into attributes for use with
    /// the renderer using the given dimensions.
    ///
    /// Easier to pass in the dimensions opposed to storing it everywhere.
    pub(crate) fn tiles_to_renderer_parts<D: Into<Dimension2>>(
        &self,
        z: usize,
        dimensions: D,
    ) -> Option<(Vec<f32>, Vec<[f32; 4]>)> {
        let dimensions = dimensions.into();
        let area = dimensions.area() as usize;
        self.sprite_layers.get(z).and_then(|o| {
            o.as_ref()
                .map(|layer| layer.inner.as_ref().tiles_to_attributes(area))
        })
    }
}

/// The chunk update system that is used to set attributes of the tiles and
/// tints if they need updating.
pub(crate) fn chunk_update_system(
    mut commands: Commands,
    chunks: Res<Assets<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(
        Entity,
        &ChunkDimensions,
        &Handle<Chunk>,
        &Handle<Mesh>,
        &DirtyLayer,
    )>,
) {
    for (entity, dimensions, chunk_handle, mesh_handle, dirty_layer) in query.iter() {
        let chunk = chunks.get(chunk_handle).expect("`Chunk` is missing");
        let mesh = meshes.get_mut(mesh_handle).expect("`Mesh` is missing");

        let (indexes, colors) = chunk
            .tiles_to_renderer_parts(dirty_layer.0, dimensions.dimensions)
            .expect("Tiles missing.");

        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes.into());
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors.into());

        commands.remove_one::<DirtyLayer>(entity);
    }
}
