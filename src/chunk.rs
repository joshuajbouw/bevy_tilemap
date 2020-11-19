use crate::{dimension::Dimension2, lib::*, point::Point2, tile::Tile};

/// A component that stores the dimensions of the Chunk for the renderer.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, RenderResources, RenderResource)]
#[render_resources(from_self)]
pub struct ChunkDimensions {
    /// The chunk dimensions.
    pub dimensions: Vec2,
}

unsafe impl Byteable for ChunkDimensions {}

impl From<Dimension2> for ChunkDimensions {
    fn from(dimensions: Dimension2) -> ChunkDimensions {
        ChunkDimensions {
            dimensions: dimensions.into(),
        }
    }
}

impl From<Vec2> for ChunkDimensions {
    fn from(vec: Vec2) -> ChunkDimensions {
        ChunkDimensions { dimensions: vec }
    }
}

pub(crate) trait Layer: 'static {
    fn mesh(&self) -> &Handle<Mesh>;

    fn set_mesh(&mut self, mesh: Handle<Mesh>);

    fn set_tile(&mut self, index: usize, tile: Tile);

    fn tiles_to_renderer_parts(&self, area: usize) -> (Vec<f32>, Vec<[f32; 4]>);
}

/// A layer with dense sprite tiles.
///
/// The difference between a dense layer and a sparse layer is simply the
/// storage types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, TypeUuid)]
#[uuid = "beea20b4-8d36-49cd-9410-ec1fb7696605"]
pub(crate) struct DenseLayer {
    /// A mesh handle.
    #[cfg_attr(feature = "serde", serde(skip))]
    mesh: Handle<Mesh>,
    /// A vector of all the tiles in the chunk.
    tiles: Vec<Tile>,
}

impl Layer for DenseLayer {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    fn set_mesh(&mut self, mesh: Handle<Mesh>) {
        self.mesh = mesh;
    }

    fn set_tile(&mut self, index: usize, tile: Tile) {
        self.tiles[index] = tile;
    }

    fn tiles_to_renderer_parts(&self, _area: usize) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::dense_tiles_to_attributes(&self.tiles)
    }
}

impl DenseLayer {
    pub(crate) fn new(tiles: Vec<Tile>) -> DenseLayer {
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
    tiles: HashMap<usize, Tile>,
}

impl Layer for SparseLayer {
    fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }

    fn set_mesh(&mut self, mesh: Handle<Mesh>) {
        self.mesh = mesh;
    }

    fn set_tile(&mut self, index: usize, tile: Tile) {
        self.tiles.insert(index, tile);
    }

    fn tiles_to_renderer_parts(&self, area: usize) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::sparse_tiles_to_attributes(area, &self.tiles)
    }
}

impl SparseLayer {
    pub(crate) fn new(tiles: HashMap<usize, Tile>) -> SparseLayer {
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
    /// Specifies to construct a dense sprite layer.
    Dense,
    /// Specifies to construct a sparse sprite layer.
    Sparse,
}

#[derive(Clone, PartialEq, Debug)]
enum LayerKindInner {
    Dense(DenseLayer),
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

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct SpriteLayer {
    inner: LayerKindInner,
    entity: Option<Entity>,
}

#[derive(Clone, PartialEq, Debug, TypeUuid)]
#[uuid = "47691827-0b89-4474-a14e-f2ea3c88320f"]
#[doc(hidden)]
pub struct Chunk {
    point: Point2,
    sprite_layers: Vec<Option<SpriteLayer>>,
    // flags: Vec<u32>, // TODO
}

impl Chunk {
    pub(crate) fn new(point: Point2, max_layers: usize) -> Chunk {
        Chunk {
            point,
            sprite_layers: vec![None; max_layers],
        }
    }

    pub(crate) fn add_layer(&mut self, kind: LayerKind, z: usize, dimensions: Dimension2) {
        match kind {
            LayerKind::Dense => {
                let tiles = vec![Tile::new(0); dimensions.area() as usize];
                self.sprite_layers[z] = Some(SpriteLayer {
                    inner: LayerKindInner::Dense(DenseLayer::new(tiles)),
                    entity: None,
                });
            }
            LayerKind::Sparse => {
                self.sprite_layers[z] = Some(SpriteLayer {
                    inner: LayerKindInner::Sparse(SparseLayer::new(HashMap::default())),
                    entity: None,
                })
            }
        }
    }

    pub(crate) fn point(&self) -> Point2 {
        self.point
    }

    pub(crate) fn move_layer(&mut self, from_z: usize, to_z: usize) {
        if self.sprite_layers[to_z].is_some() {
            panic!("Sprite layer {} unexpectedly exists.", to_z);
        }

        self.sprite_layers[to_z] = self.sprite_layers[from_z].clone();
        self.sprite_layers[from_z] = None;
    }

    pub(crate) fn remove_layer(&mut self, z_layer: usize) {
        self.sprite_layers[z_layer] = None;
    }

    pub(crate) fn set_mesh(&mut self, z_layer: usize, mesh: Handle<Mesh>) {
        let layer = self.sprite_layers[z_layer]
            .as_mut()
            .expect("`SpriteLayer` is missing.");
        layer.inner.as_mut().set_mesh(mesh);
    }

    pub(crate) fn set_tile(&mut self, z_layer: usize, index: usize, tile: Tile) {
        let layer = self.sprite_layers[z_layer]
            .as_mut()
            .expect("`SpriteLayer` is missing.");
        layer.inner.as_mut().set_tile(index, tile);
    }

    pub(crate) fn add_entity(&mut self, z_layer: usize, entity: Entity) {
        let layer = self.sprite_layers[z_layer]
            .as_mut()
            .expect("`SpriteLayer` is missing.");
        layer.entity = Some(entity);
    }

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

    pub(crate) fn tiles_to_renderer_parts(
        &self,
        z: usize,
        dimensions: Dimension2,
    ) -> Option<(Vec<f32>, Vec<[f32; 4]>)> {
        let area = dimensions.area() as usize;
        self.sprite_layers[z]
            .as_ref()
            .map(|sprite_layer| sprite_layer.inner.as_ref().tiles_to_renderer_parts(area))
    }
}
