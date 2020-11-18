use crate::{lib::*, tile::Tile};

/// A component that stores the dimensions of the Chunk for the renderer.
#[derive(Debug, Default, RenderResources, RenderResource)]
#[render_resources(from_self)]
pub struct ChunkDimensions {
    /// The chunk dimensions.
    pub dimensions: Vec3,
}

unsafe impl Byteable for ChunkDimensions {}

impl From<Vec3> for ChunkDimensions {
    fn from(vec: Vec3) -> Self {
        ChunkDimensions { dimensions: vec }
    }
}

pub(crate) trait Layer: 'static {
    fn mesh(&self) -> &Handle<Mesh>;

    fn set_mesh(&mut self, mesh: Handle<Mesh>);

    fn set_tile(&mut self, index: usize, tile: Tile);

    fn tiles_to_renderer_parts(&self, z: usize, layer_size: usize) -> (Vec<f32>, Vec<[f32; 4]>);
}

/// A layer with dense sprite tiles.
///
/// The difference between a dense layer and a sparse layer is simply the
/// storage types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, TypeUuid)]
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

    fn tiles_to_renderer_parts(&self, z: usize, layer_size: usize) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::dense_tiles_to_renderer_parts(z, layer_size, &self.tiles)
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
#[derive(Clone, Debug)]
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

    fn tiles_to_renderer_parts(&self, z: usize, layer_size: usize) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::sparse_tiles_to_renderer_parts(z, layer_size, &self.tiles)
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerKind {
    /// Specifies to construct a dense sprite layer.
    Dense,
    /// Specifies to construct a sparse sprite layer.
    Sparse,
}

#[derive(Clone, Debug)]
enum LayerKindInner {
    Dense(DenseLayer),
    Sparse(SparseLayer),
}

impl Deref for LayerKindInner {
    type Target = dyn Layer;

    fn deref(&self) -> &Self::Target {
        match self {
            LayerKindInner::Dense(ref s) => s,
            LayerKindInner::Sparse(ref s) => s,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SpriteLayer {
    inner: LayerKindInner,
    entity: Option<Entity>,
}

#[derive(Debug, TypeUuid)]
#[uuid = "47691827-0b89-4474-a14e-f2ea3c88320f"]
#[doc(hidden)]
pub struct Chunk {
    sprite_layers: Vec<Option<SpriteLayer>>,
    // flags: Vec<u32>, // TODO
    dimensions: Vec3,
}

impl Chunk {
    pub(crate) fn new(dimensions: Vec3, max_layers: usize) -> Chunk {
        Chunk {
            sprite_layers: Vec::with_capacity(max_layers),
            dimensions,
        }
    }

    pub(crate) fn max_size(&self) -> usize {
        (self.dimensions.x() * self.dimensions.y() * self.dimensions.z()) as usize
    }

    pub(crate) fn layer_size(&self) -> usize {
        (self.dimensions.x() * self.dimensions.y()) as usize
    }

    pub(crate) fn add_layer(&mut self, kind: LayerKind, z: usize) {
        let tiles = vec![Tile::new(0); self.max_size()];

        match kind {
            LayerKind::Dense => self.sprite_layers.insert(
                z,
                Some(SpriteLayer {
                    inner: LayerKindInner::Dense(DenseLayer::new(tiles)),
                    entity: None,
                }),
            ),
            LayerKind::Sparse => self.sprite_layers.insert(
                z,
                Some(SpriteLayer {
                    inner: LayerKindInner::Sparse(SparseLayer::new(HashMap::default())),
                    entity: None,
                }),
            ),
        }
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
        let layer = self.sprite_layers[z_layer].as_mut().unwrap();
        layer.inner.set_mesh(mesh);
    }

    pub(crate) fn set_tile(&mut self, z_layer: usize, index: usize, tile: Tile) {
        let layer = self.sprite_layers[z_layer].as_mut().unwrap();
        layer.inner.set_tile(index, tile);
    }

    pub(crate) fn add_entity(&mut self, z_layer: usize, entity: Entity) {
        let layer = self.sprite_layers[z_layer].as_mut().unwrap();
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

    pub(crate) fn tiles_to_renderer_parts(&self, z: usize) -> Option<(Vec<f32>, Vec<[f32; 4]>)> {
        self.sprite_layers[z].as_ref().map(|sprite_layer| {
            sprite_layer
                .inner
                .tiles_to_renderer_parts(z, self.layer_size())
        })
    }
}
