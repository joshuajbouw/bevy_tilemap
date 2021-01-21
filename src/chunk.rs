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
//! use bevy_asset::{prelude::*, HandleId};
//! use bevy_sprite::prelude::*;
//! use bevy_tilemap::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
//!
//! // There are two ways to create a new chunk. Either directly...
//!
//! tilemap.insert_chunk((0, 0));
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
//! use bevy_asset::{prelude::*, HandleId};
//! use bevy_sprite::prelude::*;
//! use bevy_tilemap::prelude::*;
//!
//! // This must be set in Asset<TextureAtlas>.
//! let texture_atlas_handle = Handle::weak(HandleId::random::<TextureAtlas>());
//!
//! let mut tilemap = Tilemap::new(texture_atlas_handle, 32, 32);
//!
//! tilemap.insert_chunk((0, 0));
//!
//! let z_order = 0;
//! tilemap.add_layer_with_kind(LayerKind::Dense, 0);
//!
//! let z_order = 1;
//! tilemap.add_layer_with_kind(LayerKind::Sparse, 1);
//! ```
use crate::{
    entity::{ModifiedLayer, ZOrder},
    lib::*,
    mesh::ChunkMesh,
    prelude::Tile,
    tile::RawTile,
    tilemap::Tilemap,
};

/// Common methods for layers in a chunk.
pub(crate) trait Layer: 'static {
    /// Returns the handle of the mesh.
    fn mesh(&self) -> &Handle<Mesh>;

    /// Sets the mesh for the layer.
    fn set_mesh(&mut self, mesh: Handle<Mesh>);

    /// Sets a raw tile for a layer at an index.
    fn set_tile(&mut self, index: usize, tile: RawTile);

    /// Removes a tile for a layer at an index.
    fn remove_tile(&mut self, index: usize);

    /// Gets a tile by an index.
    fn get_tile(&self, index: usize) -> Option<&RawTile>;

    /// Gets a tile with a mutable reference by an index.
    fn get_tile_mut(&mut self, index: usize) -> Option<&mut RawTile>;

    /// Gets all the tile indices in the layer that exist.
    fn get_tile_indices(&self) -> Vec<usize>;

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

    fn set_tile(&mut self, index: usize, tile: RawTile) {
        if let Some(inner_tile) = self.tiles.get_mut(index) {
            *inner_tile = tile;
        } else {
            warn!(
                "tile is out of bounds at index {} and can not be set",
                index
            );
        }
    }

    fn remove_tile(&mut self, index: usize) {
        if let Some(tile) = self.tiles.get_mut(index) {
            tile.color.set_a(0.0);
        }
    }

    fn get_tile(&self, index: usize) -> Option<&RawTile> {
        self.tiles.get(index).and_then(|tile| {
            if tile.color.a() == 0.0 {
                None
            } else {
                Some(tile)
            }
        })
    }

    fn get_tile_mut(&mut self, index: usize) -> Option<&mut RawTile> {
        self.tiles.get_mut(index).and_then(|tile| {
            if tile.color.a() == 0.0 {
                None
            } else {
                Some(tile)
            }
        })
    }

    fn get_tile_indices(&self) -> Vec<usize> {
        let mut indices = Vec::with_capacity(self.tiles.len());
        for (index, tile) in self.tiles.iter().enumerate() {
            if tile.color.a() != 0.0 {
                indices.push(index);
            }
        }
        indices.shrink_to_fit();
        indices
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

    fn set_tile(&mut self, index: usize, tile: RawTile) {
        if tile.color.a() == 0.0 {
            self.tiles.remove(&index);
        }
        self.tiles.insert(index, tile);
    }

    fn remove_tile(&mut self, index: usize) {
        self.tiles.remove(&index);
    }

    fn get_tile(&self, index: usize) -> Option<&RawTile> {
        self.tiles.get(&index)
    }

    fn get_tile_mut(&mut self, index: usize) -> Option<&mut RawTile> {
        self.tiles.get_mut(&index)
    }

    fn get_tile_indices(&self) -> Vec<usize> {
        let mut indices = Vec::with_capacity(self.tiles.len());
        for index in self.tiles.keys() {
            indices.push(*index);
        }
        indices
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
    /// Contains a map of all collision entities.
    collision_entities: HashMap<usize, Entity>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
#[doc(hidden)]
pub struct Chunk {
    point: Point2,
    sprite_layers: Vec<Option<SpriteLayer>>,
}

impl Chunk {
    /// A newly constructed chunk from a point and the maximum number of layers.
    pub(crate) fn new(
        point: Point2,
        layers: &[Option<LayerKind>],
        dimensions: Dimension2,
    ) -> Chunk {
        let mut chunk = Chunk {
            point,
            sprite_layers: vec![None; layers.len()],
        };
        for (z_order, kind) in layers.iter().enumerate() {
            if let Some(kind) = kind {
                chunk.add_layer(kind, z_order, dimensions)
            }
        }
        chunk
    }

    /// Adds a layer from a layer kind, the z layer, and dimensions of the
    /// chunk.
    pub(crate) fn add_layer(&mut self, kind: &LayerKind, z_order: usize, dimensions: Dimension2) {
        match kind {
            LayerKind::Dense => {
                let tiles = vec![
                    RawTile {
                        index: 0,
                        color: Color::rgba(0.0, 0.0, 0.0, 0.0)
                    };
                    dimensions.area() as usize
                ];
                if let Some(layer) = self.sprite_layers.get_mut(z_order) {
                    *layer = Some(SpriteLayer {
                        inner: LayerKindInner::Dense(DenseLayer::new(tiles)),
                        entity: None,
                        collision_entities: HashMap::default(),
                    });
                } else {
                    error!("sprite layer {} is out of bounds", z_order);
                }
            }
            LayerKind::Sparse => {
                if let Some(layer) = self.sprite_layers.get_mut(z_order) {
                    *layer = Some(SpriteLayer {
                        inner: LayerKindInner::Sparse(SparseLayer::new(HashMap::default())),
                        entity: None,
                        collision_entities: HashMap::default(),
                    });
                } else {
                    error!("sprite layer {} is out of bounds", z_order);
                }
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
            error!(
                "sprite layer {} unexpectedly exists and can not be moved",
                to_z
            );
            return;
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
            } else {
                error!("can not set mesh to sprite layer {}", z_order);
            }
        } else {
            error!("sprite layer {} does not exist", z_order);
        }
    }

    /// Sets a single raw tile to be added to a z layer and index.
    pub(crate) fn set_tile(&mut self, index: usize, tile: Tile) {
        if let Some(layer) = self.sprite_layers.get_mut(tile.z_order) {
            if let Some(layer) = layer.as_mut() {
                let raw_tile = RawTile {
                    index: tile.sprite_index,
                    color: tile.tint,
                };
                layer.inner.as_mut().set_tile(index, raw_tile);
            } else {
                error!("can not set tile to sprite layer {}", tile.z_order);
            }
        } else {
            error!("sprite layer {} does not exist", tile.z_order);
        }
    }

    /// Removes a tile from a sprite layer with a given index and z order.
    pub(crate) fn remove_tile(&mut self, index: usize, z_order: usize) {
        if let Some(layer) = self.sprite_layers.get_mut(z_order) {
            if let Some(layer) = layer.as_mut() {
                layer.inner.as_mut().remove_tile(index);
            } else {
                error!("can not remove tile on sprite layer {}", z_order);
            }
        } else {
            error!("sprite layer {} does not exist", z_order);
        }
    }

    /// Adds an entity to a z layer, always when it is spawned.
    pub(crate) fn add_entity(&mut self, z_order: usize, entity: Entity) {
        if let Some(layer) = self.sprite_layers.get_mut(z_order) {
            if let Some(layer) = layer.as_mut() {
                layer.entity = Some(entity);
            } else {
                error!("can not add entity to sprite layer {}", z_order);
            }
        } else {
            error!("sprite layer {} does not exist", z_order);
        }
    }

    /// Adds an entity to a tile index in a layer.
    pub(crate) fn insert_collision_entity(&mut self, z_order: usize, index: usize, entity: Entity) {
        if let Some(layer) = self.sprite_layers.get_mut(z_order) {
            if let Some(layer) = layer.as_mut() {
                layer.collision_entities.insert(index, entity);
            } else {
                error!("can not add collision entity to sprite layer {}", z_order);
            }
        } else {
            error!("sprite layer {} does not exist", z_order);
        }
    }

    /// Gets the layers entity, if any. Useful for despawning.
    pub(crate) fn get_entity(&self, z_order: usize) -> Option<Entity> {
        self.sprite_layers
            .get(z_order)
            .and_then(|o| o.as_ref().and_then(|layer| layer.entity))
    }

    /// Gets the collision entity if any.
    pub(crate) fn get_collision_entities(&self, index: usize, z_order: usize) -> Option<Entity> {
        self.sprite_layers.get(z_order).and_then(|o| {
            o.as_ref()
                .and_then(|layer| layer.collision_entities.get(&index).cloned())
        })
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

    /// Gets a reference to a tile from a provided z order and index.
    pub(crate) fn get_tile(&self, z_order: usize, index: usize) -> Option<&RawTile> {
        self.sprite_layers.get(z_order).and_then(|layer| {
            layer
                .as_ref()
                .and_then(|layer| layer.inner.as_ref().get_tile(index))
        })
    }

    /// Gets a mutable reference to a tile from a provided z order and index.
    pub(crate) fn get_tile_mut(&mut self, z_order: usize, index: usize) -> Option<&mut RawTile> {
        self.sprite_layers.get_mut(z_order).and_then(|layer| {
            layer
                .as_mut()
                .and_then(|layer| layer.inner.as_mut().get_tile_mut(index))
        })
    }

    /// Gets a vec of all the tiles in the layer, if any.
    pub(crate) fn get_tile_indices(&self, z_order: usize) -> Option<Vec<usize>> {
        self.sprite_layers.get(z_order).and_then(|layer| {
            layer
                .as_ref()
                .map(|layer| layer.inner.as_ref().get_tile_indices())
        })
    }

    /// At the given z layer, changes the tiles into attributes for use with
    /// the renderer using the given dimensions.
    ///
    /// Easier to pass in the dimensions opposed to storing it everywhere.
    pub(crate) fn tiles_to_renderer_parts(
        &self,
        z: usize,
        dimensions: Dimension2,
    ) -> Option<(Vec<f32>, Vec<[f32; 4]>)> {
        let area = dimensions.area() as usize;
        self.sprite_layers.get(z).and_then(|o| {
            o.as_ref()
                .map(|layer| layer.inner.as_ref().tiles_to_attributes(area))
        })
    }
}

/// The chunk update system that is used to set attributes of the tiles and
/// tints if they need updating.
pub(crate) fn chunk_update(
    mut meshes: ResMut<Assets<Mesh>>,
    map_query: Query<&Tilemap>,
    mut chunk_query: Query<(&Parent, &Point2, &ZOrder, &Handle<Mesh>), Changed<ModifiedLayer>>,
) {
    for (parent, point, z_order, mesh_handle) in chunk_query.iter_mut() {
        let tilemap = if let Ok(tilemap) = map_query.get(**parent) {
            tilemap
        } else {
            error!("`Tilemap` is missing, can not update chunk");
            return;
        };
        let chunk = if let Some(chunk) = tilemap.get_chunk(point) {
            chunk
        } else {
            error!("`Chunk` is missing, can not update chunk");
            return;
        };
        let mesh = if let Some(mesh) = meshes.get_mut(mesh_handle) {
            mesh
        } else {
            error!("`Mesh` is missing, can not update chunk");
            return;
        };
        let (indexes, colors) = if let Some((index, colors)) =
            chunk.tiles_to_renderer_parts(z_order.0, tilemap.chunk_dimensions())
        {
            (index, colors)
        } else {
            error!("Tiles are missing, can not update chunk");
            return;
        };
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_INDEX, indexes);
        mesh.set_attribute(ChunkMesh::ATTRIBUTE_TILE_COLOR, colors);
    }
}

/// Actual method used to spawn chunks.
fn auto_spawn(
    camera_transform: &Transform,
    tilemap_transform: &Transform,
    tilemap: &mut Tilemap,
    spawn_dimensions: Dimension2,
) {
    let translation = camera_transform.translation - tilemap_transform.translation;
    let point_x = translation.x / tilemap.tile_width() as f32;
    let point_y = translation.y / tilemap.tile_height() as f32;
    let (chunk_x, chunk_y) = tilemap.point_to_chunk_point((point_x as i32, point_y as i32));
    let mut new_spawned: Vec<Point2> = Vec::new();
    let spawn_width = spawn_dimensions.width as i32;
    let spawn_height = spawn_dimensions.height as i32;
    for y in -spawn_width as i32..spawn_width + 1 {
        for x in -spawn_height..spawn_height + 1 {
            let chunk_x = x + chunk_x;
            let chunk_y = y + chunk_y;
            if let Some(width) = tilemap.width() {
                let width = (width / tilemap.chunk_width()) as i32 / 2;
                if chunk_x < -width || chunk_x > width {
                    continue;
                }
            }
            if let Some(height) = tilemap.height() {
                let height = (height / tilemap.chunk_height()) as i32 / 2;
                if chunk_y < -height || chunk_y > height {
                    continue;
                }
            }

            if let Err(e) = tilemap.spawn_chunk(Point2::new(chunk_x, chunk_y)) {
                warn!("{}", e);
            }
            new_spawned.push(Point2::new(chunk_x, chunk_y));
        }
    }

    let spawned_list = tilemap.spawned_chunks_mut().clone();
    for point in spawned_list.iter() {
        if !new_spawned.contains(&point.into()) {
            if let Err(e) = tilemap.despawn_chunk(point) {
                warn!("{}", e);
            }
        }
    }
}

/// On window size change, the radius of chunks changes if needed.
pub(crate) fn chunk_auto_radius(
    window_resized_events: Res<Events<WindowResized>>,
    mut tilemap_query: Query<(&mut Tilemap, &Transform)>,
    camera_query: Query<(&Camera, &Transform)>,
) {
    let mut window_reader = window_resized_events.get_reader();
    for event in window_reader.iter(&window_resized_events) {
        for (mut tilemap, tilemap_transform) in tilemap_query.iter_mut() {
            let window_width = event.width as u32;
            let window_height = event.height as u32;
            let chunk_px_width = tilemap.chunk_width() * tilemap.tile_width();
            let chunk_px_height = tilemap.chunk_height() * tilemap.tile_height();
            let chunks_wide = (window_width as f32 / chunk_px_width as f32).ceil() as u32 + 1;
            let chunks_high = (window_height as f32 / chunk_px_height as f32).ceil() as u32 + 1;
            let spawn_dimensions = Dimension2::new(chunks_wide, chunks_high);
            tilemap.set_auto_spawn(spawn_dimensions);
            for (_camera, camera_transform) in camera_query.iter() {
                auto_spawn(
                    camera_transform,
                    &tilemap_transform,
                    &mut tilemap,
                    spawn_dimensions,
                );
            }
        }
    }
}

/// Spawns and despawns chunks automatically based on a camera's position.
pub(crate) fn chunk_auto_spawn(
    mut tilemap_query: Query<(&mut Tilemap, &Transform)>,
    camera_query: Query<(&Camera, &Transform), Changed<Transform>>,
) {
    // For the transform, get chunk coord.
    for (mut tilemap, tilemap_transform) in tilemap_query.iter_mut() {
        for (_camera, camera_transform) in camera_query.iter() {
            let spawn_dimensions = if let Some(dimensions) = tilemap.auto_spawn() {
                dimensions
            } else {
                continue;
            };
            auto_spawn(
                camera_transform,
                &tilemap_transform,
                &mut tilemap,
                spawn_dimensions,
            );
        }
    }
}
