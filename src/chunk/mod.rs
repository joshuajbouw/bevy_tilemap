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
//! let z_order = 0;
//! tilemap.add_layer_with_kind(LayerKind::Dense, 0);
//!
//! let z_order = 1;
//! tilemap.add_layer_with_kind(LayerKind::Sparse, 1);
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
#[derive(Clone, PartialEq, Debug)]
#[doc(hidden)]
pub(crate) struct Chunk {
    /// The point coordinate of the chunk.
    point: Point2,
    /// The sprite layers of the chunk.
    sprite_layers: Vec<Option<SpriteLayer>>,
    /// Ephemeral user data that can be used for flags or other purposes.
    user_data: u128,
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
            user_data: 0,
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

    // /// Returns a copy of the user data.
    // pub(crate) fn user_data(&self) -> u128 {
    //     self.user_data
    // }
    //
    // /// Returns a mutable reference to the user data.
    // pub(crate) fn user_data_mut(&mut self) -> &mut u128 {
    //     &mut self.user_data
    // }

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
    pub(crate) fn set_tile<P: Into<Point2>>(&mut self, index: usize, tile: Tile<P>) {
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
