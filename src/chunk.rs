use crate::{
    dimension::Dimension2,
    entity::{ChunkDimensions, DirtyLayer},
    lib::*,
    mesh::ChunkMesh,
    point::Point2,
    tile::RawTile,
};

pub(crate) trait Layer: 'static {
    fn mesh(&self) -> &Handle<Mesh>;

    fn set_mesh(&mut self, mesh: Handle<Mesh>);

    fn set_raw_tile(&mut self, index: usize, tile: RawTile);

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
        self.tiles[index] = tile;
    }

    fn tiles_to_renderer_parts(&self, _area: usize) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::dense_tiles_to_attributes(&self.tiles)
    }
}

impl DenseLayer {
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

    fn tiles_to_renderer_parts(&self, area: usize) -> (Vec<f32>, Vec<[f32; 4]>) {
        crate::tile::sparse_tiles_to_attributes(area, &self.tiles)
    }
}

impl SparseLayer {
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
    /// Specifies to construct a dense sprite layer.
    Dense,
    /// Specifies to construct a sparse sprite layer.
    Sparse,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct SpriteLayer {
    inner: LayerKindInner,
    #[cfg_attr(feature = "serde", serde(skip))]
    entity: Option<Entity>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    pub(crate) fn remove_layer(&mut self, z_order: usize) {
        self.sprite_layers[z_order] = None;
    }

    pub(crate) fn set_mesh(&mut self, z_order: usize, mesh: Handle<Mesh>) {
        let layer = self.sprite_layers[z_order]
            .as_mut()
            .expect("`SpriteLayer` is missing.");
        layer.inner.as_mut().set_mesh(mesh);
    }

    pub(crate) fn set_raw_tile(&mut self, z_order: usize, index: usize, raw_tile: RawTile) {
        let layer = self.sprite_layers[z_order]
            .as_mut()
            .expect("`SpriteLayer` is missing.");
        layer.inner.as_mut().set_raw_tile(index, raw_tile);
    }

    pub(crate) fn add_entity(&mut self, z_order: usize, entity: Entity) {
        let layer = self.sprite_layers[z_order]
            .as_mut()
            .expect("`SpriteLayer` is missing.");
        layer.entity = Some(entity);
    }

    pub(crate) fn get_entity(&self, z_order: usize) -> Option<Entity> {
        let layer = self.sprite_layers[z_order]
            .as_ref()
            .expect("`SpriteLayer` is missing.");
        layer.entity
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

    pub(crate) fn tiles_to_renderer_parts<D: Into<Dimension2>>(
        &self,
        z: usize,
        dimensions: D,
    ) -> Option<(Vec<f32>, Vec<[f32; 4]>)> {
        let dimensions = dimensions.into();
        let area = dimensions.area() as usize;
        self.sprite_layers[z]
            .as_ref()
            .map(|sprite_layer| sprite_layer.inner.as_ref().tiles_to_renderer_parts(area))
    }
}

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
