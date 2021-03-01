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
//! let tile = Tile { point, sprite_index, ..Default::default() };
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
//! let sprite_order = 0;
//! tilemap.add_layer(TilemapLayer { kind: LayerKind::Dense, ..Default::default() }, 1);
//!
//! let sprite_order = 1;
//! tilemap.add_layer(TilemapLayer { kind: LayerKind::Dense, ..Default::default() }, 1);
//! ```

/// Chunk entity.
pub(crate) mod entity;
/// Sparse and dense chunk layers.
mod layer;
/// Meshes for rendering to vertices.
pub(crate) mod mesh;
/// Raw tile that is stored in the chunks.
pub mod raw_tile;
/// Files and helpers for rendering.
pub(crate) mod render;
/// Systems for chunks.
pub(crate) mod system;

use crate::{lib::*, tile::Tile};
pub use layer::LayerKind;
use layer::{DenseLayer, LayerKindInner, SparseLayer, SpriteLayer};
pub use raw_tile::RawTile;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// A chunk which holds all the tiles to be rendered.
pub(crate) struct Chunk {
    /// The point coordinate of the chunk.
    point: Point2,
    /// The sprite layers of the chunk.
    z_layers: Vec<Vec<SpriteLayer>>,
    /// Ephemeral user data that can be used for flags or other purposes.
    user_data: u128,
    /// A chunks mesh used for rendering.
    #[cfg_attr(feature = "serde", serde(skip))]
    mesh: Option<Handle<Mesh>>,
    /// An entity which is tied to this chunk.
    entity: Option<Entity>,
}

impl Chunk {
    /// A newly constructed chunk from a point and the maximum number of layers.
    pub(crate) fn new(
        point: Point2,
        layers: &[Option<LayerKind>],
        dimensions: Dimension3,
    ) -> Chunk {
        let mut chunk = Chunk {
            point,
            z_layers: vec![vec![
                SpriteLayer {
                    inner: LayerKindInner::Sparse(SparseLayer::new(HashMap::default())),
                    entity: None,
                };
                layers.len()
            ]],
            user_data: 0,
            mesh: None,
            entity: None,
        };

        for (sprite_order, kind) in layers.iter().enumerate() {
            if let Some(kind) = kind {
                chunk.add_layer(kind, sprite_order, dimensions)
            }
        }

        chunk
    }

    /// Adds a layer from a layer kind, the z layer, and dimensions of the
    /// chunk.
    pub(crate) fn add_layer(
        &mut self,
        kind: &LayerKind,
        sprite_order: usize,
        dimensions: Dimension3,
    ) {
        for z in 0..dimensions.depth as usize {
            match kind {
                LayerKind::Dense => {
                    let tiles = vec![
                        RawTile {
                            index: 0,
                            color: Color::rgba(0.0, 0.0, 0.0, 0.0)
                        };
                        (dimensions.width * dimensions.height) as usize
                    ];
                    if let Some(z_layer) = self.z_layers.get_mut(z) {
                        if let Some(sprite_order_layer) = z_layer.get_mut(sprite_order) {
                            *sprite_order_layer = SpriteLayer {
                                inner: LayerKindInner::Dense(DenseLayer::new(tiles)),
                                entity: None,
                            };
                        } else {
                            error!("sprite layer {} could not be added?", sprite_order);
                        }
                    } else {
                        error!("sprite layer {} is out of bounds", sprite_order);
                    }
                    info!("Set a new dense layer at layer: {}", sprite_order);
                }
                LayerKind::Sparse => {
                    if let Some(z_layer) = self.z_layers.get_mut(z) {
                        if let Some(sprite_order_layer) = z_layer.get_mut(sprite_order) {
                            *sprite_order_layer = SpriteLayer {
                                inner: LayerKindInner::Sparse(SparseLayer::new(HashMap::default())),
                                entity: None,
                            };
                        } else {
                            error!("sprite layer {} is out of bounds", sprite_order);
                        }
                    } else {
                        error!("sprite layer {} is out of bounds", sprite_order);
                    }
                    info!("Set a new sparse layer at layer: {}", sprite_order);
                }
            }
        }
        info!("LAYERS len: {}", self.z_layers.len());
    }

    /// Returns the point of the location of the chunk.
    pub(crate) fn point(&self) -> Point2 {
        self.point
    }

    /// Moves a layer from a z layer to another.
    pub(crate) fn move_sprite_order(&mut self, from_sprite_order: usize, to_sprite_order: usize) {
        for z in 0..self.z_layers.len() {
            if self.z_layers.get(from_sprite_order).is_some() {
                error!(
                    "sprite layer {} exists and can not be moved",
                    to_sprite_order
                );
                return;
            }
            if let Some(sprite_layer) = self.z_layers.get_mut(z) {
                sprite_layer.swap(from_sprite_order, to_sprite_order);
            }
        }
    }

    /// Removes a layer from the specified layer.
    pub(crate) fn remove_layer(&mut self, sprite_order: usize) {
        for _z in 0..self.z_layers.len() {
            self.z_layers.get_mut(sprite_order).take();
        }
    }

    /// Sets the mesh for the chunk layer to use.
    pub(crate) fn set_mesh(&mut self, mesh: Handle<Mesh>) {
        self.mesh = Some(mesh);
    }

    /// Takes the mesh handle.
    pub(crate) fn take_mesh(&mut self) -> Option<Handle<Mesh>> {
        self.mesh.take()
    }

    /// Sets a single raw tile to be added to a z layer and index.
    pub(crate) fn set_tile(&mut self, index: usize, tile: Tile<Point3>) {
        if let Some(z_depth) = self.z_layers.get_mut(tile.point.z as usize) {
            if let Some(layer) = z_depth.get_mut(tile.sprite_order) {
                let raw_tile = RawTile {
                    index: tile.sprite_index,
                    color: tile.tint,
                };
                layer.inner.as_mut().set_tile(index, raw_tile);
            } else {
                error!("sprite layer {} does not exist", tile.sprite_order);
            }
        } else {
            error!("z layer {} does not exist", tile.point.z);
        }
    }

    /// Removes a tile from a sprite layer with a given index and z order.
    pub(crate) fn remove_tile(&mut self, index: usize, sprite_order: usize, z_depth: usize) {
        if let Some(z_depth) = self.z_layers.get_mut(z_depth) {
            if let Some(layer) = z_depth.get_mut(sprite_order) {
                layer.inner.as_mut().remove_tile(index);
            } else {
                error!("can not remove tile on sprite layer {}", sprite_order);
            }
        } else {
            error!("sprite layer {} does not exist", sprite_order);
        }
    }

    /// Adds an entity to a z layer, always when it is spawned.
    pub(crate) fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    /// Gets the mesh entity of the chunk.
    pub(crate) fn get_entity(&self) -> Option<Entity> {
        self.entity
    }

    /// Gets the layers entity, if any. Useful for despawning.
    pub(crate) fn take_entity(&mut self) -> Option<Entity> {
        self.entity.take()
    }

    /// Gets a reference to a tile from a provided z order and index.
    pub(crate) fn get_tile(
        &self,
        index: usize,
        sprite_order: usize,
        z_depth: usize,
    ) -> Option<&RawTile> {
        self.z_layers.get(z_depth).and_then(|z_depth| {
            z_depth
                .get(sprite_order)
                .and_then(|layer| layer.inner.as_ref().get_tile(index))
        })
    }

    /// Gets a mutable reference to a tile from a provided z order and index.
    pub(crate) fn get_tile_mut(
        &mut self,
        index: usize,
        sprite_order: usize,
        z_depth: usize,
    ) -> Option<&mut RawTile> {
        self.z_layers.get_mut(z_depth).and_then(|z_depth| {
            z_depth
                .get_mut(sprite_order)
                .and_then(|layer| layer.inner.as_mut().get_tile_mut(index))
        })
    }

    /// At the given z layer, changes the tiles into attributes for use with
    /// the renderer using the given dimensions.
    ///
    /// Easier to pass in the dimensions opposed to storing it everywhere.
    pub(crate) fn tiles_to_renderer_parts(
        &self,
        dimensions: Dimension3,
    ) -> (Vec<f32>, Vec<[f32; 4]>) {
        let mut tile_indices = Vec::new();
        let mut tile_colors = Vec::new();
        for depth in &self.z_layers {
            for layer in depth {
                let (mut indices, mut colors) =
                    layer.inner.as_ref().tiles_to_attributes(dimensions);
                tile_indices.append(&mut indices);
                tile_colors.append(&mut colors);
            }
        }
        (tile_indices, tile_colors)
    }
}
