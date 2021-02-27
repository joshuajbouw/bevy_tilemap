use crate::{chunk::raw_tile::RawTile, lib::*};

/// Common methods for layers in a chunk.
pub(super) trait Layer: 'static {
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
    fn tiles_to_attributes(&self, dimension: Dimension3) -> (Vec<f32>, Vec<[f32; 4]>);
}

/// A layer with dense sprite tiles.
///
/// The difference between a dense layer and a sparse layer is simply the
/// storage types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub(super) struct DenseLayer {
    /// A vector of all the tiles in the chunk.
    tiles: Vec<RawTile>,
    /// A count of the tiles to keep track if layer is empty or not.
    tile_count: usize,
}

impl Layer for DenseLayer {
    fn set_tile(&mut self, index: usize, tile: RawTile) {
        if let Some(inner_tile) = self.tiles.get_mut(index) {
            self.tile_count += 1;
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
            if self.tile_count != 0 {
                self.tile_count -= 1;
                tile.color.set_a(0.0);
            }
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

    fn tiles_to_attributes(&self, _dimension: Dimension3) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::chunk::raw_tile::dense_tiles_to_attributes(&self.tiles)
    }
}

impl DenseLayer {
    /// Constructs a new dense layer with tiles.
    pub fn new(tiles: Vec<RawTile>) -> DenseLayer {
        DenseLayer {
            tiles,
            tile_count: 0,
        }
    }
}

/// A layer with sparse sprite tiles.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub(super) struct SparseLayer {
    /// A map of all the tiles in the chunk.
    tiles: HashMap<usize, RawTile>,
}

impl Layer for SparseLayer {
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

    fn tiles_to_attributes(&self, dimension: Dimension3) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::chunk::raw_tile::sparse_tiles_to_attributes(dimension, &self.tiles)
    }
}

impl SparseLayer {
    /// Constructs a new sparse layer with a tile hashmap.
    pub fn new(tiles: HashMap<usize, RawTile>) -> SparseLayer {
        SparseLayer { tiles }
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
pub(super) enum LayerKindInner {
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
pub(super) struct SpriteLayer {
    /// Enum storage of the kind of layer.
    pub inner: LayerKindInner,
    #[cfg_attr(feature = "serde", serde(skip))]
    /// Contains an entity if the layer had been spawned.
    pub entity: Option<Entity>,
}
