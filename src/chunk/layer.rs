use crate::{lib::*, Tile};

/// Common methods for layers in a chunk.
pub(super) trait Layer: 'static {
    /// Sets a raw tile for a layer at an index.
    fn set_tile(&mut self, index: usize, tile: Entity);

    /// Removes a tile for a layer at an index.
    fn remove_tile(&mut self, index: usize) -> Option<Entity>;

    /// Gets a tile by an index.
    fn get_tile(&self, index: usize) -> Option<Entity>;

    /// Gets all the tile indices in the layer that exist.
    fn get_tile_indices(&self) -> Vec<usize>;

    /// Clears a layer of all sprites.
    fn clear(&mut self, commands: &mut Commands);

    /// Takes all the tiles in the layer and returns attributes for the renderer.
    fn tiles_to_attributes(
        &self,
        tile_query: &Query<&Tile<Point3>>,
        dimension: Dimension3,
    ) -> (Vec<f32>, Vec<[f32; 4]>);
}

/// A layer with dense sprite tiles.
///
/// The difference between a dense layer and a sparse layer is simply the
/// storage types.
#[derive(Component, Clone, Default, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component, PartialEq, Serialize, Deserialize)]
pub(super) struct DenseLayer {
    /// A vector of all the tiles in the chunk.
    tiles: Vec<Option<Entity>>,
    /// A count of the tiles to keep track if layer is empty or not.
    tile_count: usize,
}

impl Layer for DenseLayer {
    fn set_tile(&mut self, index: usize, tile: Entity) {
        if let Some(inner_tile) = self.tiles.get_mut(index) {
            self.tile_count += 1;
            *inner_tile = Some(tile);
        } else {
            warn!(
                "tile is out of bounds at index {} and can not be set",
                index
            );
        }
    }

    fn remove_tile(&mut self, index: usize) -> Option<Entity> {
        let maybe_entity = self.tiles.remove(index);

        if maybe_entity.is_some() && self.tile_count != 0 {
            self.tile_count -= 1;
        }

        maybe_entity
    }

    fn get_tile(&self, index: usize) -> Option<Entity> {
        self.tiles.get(index).and_then(|tile| *tile)
    }

    fn get_tile_indices(&self) -> Vec<usize> {
        let mut indices = Vec::with_capacity(self.tiles.len());
        for (index, tile) in self.tiles.iter().enumerate() {
            if tile.is_some() {
                indices.push(index);
            }
        }
        indices.shrink_to_fit();
        indices
    }

    fn clear(&mut self, commands: &mut Commands) {
        for entity in self.tiles.iter().flatten() {
            commands.entity(*entity).despawn();
        }
        self.tiles.clear();
    }

    fn tiles_to_attributes(
        &self,
        tile_query: &Query<&Tile<Point3>>,
        _dimension: Dimension3,
    ) -> (Vec<f32>, Vec<[f32; 4]>) {
        let mut tiles: Vec<&Tile<Point3>> = Vec::with_capacity(self.tiles.len());
        for entity in self.tiles.iter().flatten() {
            let tile: &Tile<Point3> = tile_query.get(*entity).expect("Can't fail");
            tiles.push(tile);
        }

        crate::chunk::dense_tiles_to_attributes(tiles)
    }
}

impl DenseLayer {
    /// Constructs a new dense layer with tiles.
    pub fn new(tiles: Vec<Option<Entity>>) -> DenseLayer {
        DenseLayer {
            tiles,
            tile_count: 0,
        }
    }
}

/// A layer with sparse sprite tiles.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub(super) struct SparseLayer {
    /// A map of all the tiles in the chunk.
    tiles: HashMap<usize, Entity>,
}

impl Layer for SparseLayer {
    fn set_tile(&mut self, index: usize, tile: Entity) {
        self.tiles.insert(index, tile);
    }

    fn remove_tile(&mut self, index: usize) -> Option<Entity> {
        self.tiles.remove(&index)
    }

    fn get_tile(&self, index: usize) -> Option<Entity> {
        self.tiles.get(&index).cloned()
    }

    fn get_tile_indices(&self) -> Vec<usize> {
        let mut indices = Vec::with_capacity(self.tiles.len());
        for index in self.tiles.keys() {
            indices.push(*index);
        }
        indices
    }

    fn clear(&mut self, commands: &mut Commands) {
        for (_, entity) in self.tiles.iter() {
            commands.entity(*entity).despawn();
        }
        self.tiles.clear();
    }

    fn tiles_to_attributes(
        &self,
        tile_query: &Query<&Tile<Point3>>,
        dimension: Dimension3,
    ) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::chunk::sparse_tiles_to_attributes(tile_query, dimension, &self.tiles)
    }
}

impl SparseLayer {
    /// Constructs a new sparse layer with a tile hashmap.
    pub fn new(tiles: HashMap<usize, Entity>) -> SparseLayer {
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
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize, Reflect,
)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum LayerKind {
    /// Specifies the tilemap to add a dense sprite layer.
    Dense,
    /// Specifies the tilemap to add a sparse sprite layer.
    Sparse,
}

#[derive(Component, Clone, PartialEq, Debug, Serialize, Deserialize, Reflect)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
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

#[derive(Component, Clone, PartialEq, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component, PartialEq, Serialize, Deserialize)]
/// A sprite layer which can either store a sparse or dense layer.
pub(super) struct SpriteLayer {
    /// Enum storage of the kind of layer.
    pub inner: LayerKindInner,
}

impl Default for SpriteLayer {
    fn default() -> Self {
        SpriteLayer {
            inner: LayerKindInner::Dense(DenseLayer::default()),
        }
    }
}
