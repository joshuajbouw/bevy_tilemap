use crate::{
    lib::{Bundle, *},
    Tilemap,
};

/// A component that is used as a flag for dirty chunks that need updating.
#[derive(Default)]
pub(crate) struct ModifiedLayer(pub(crate) usize);

/// The Z Order of a layer in a chunk.
pub(crate) struct ZOrder(pub(crate) usize);

/// A component bundle for `Chunk` entities.
#[derive(Bundle)]
pub(crate) struct ChunkBundle {
    /// The point of the chunk.
    pub(crate) point: Point2,
    /// The z order of the layer.
    pub(crate) z_order: ZOrder,
    /// The handle for a TextureAtlas which contains multiple textures.
    pub(crate) texture_atlas: Handle<TextureAtlas>,
    /// A component that indicates how to draw a component.
    pub(crate) draw: Draw,
    /// A component that indicates if the component is visible.
    pub(crate) visible: Visible,
    /// The pipeline for the renderer.
    pub(crate) render_pipelines: RenderPipelines,
    /// A component that indicates that an entity should be drawn in the
    /// "main pass"
    pub(crate) main_pass: MainPass,
    /// A mesh of vertices for a component.
    pub(crate) mesh: Handle<Mesh>,
    /// The transform location in a space for a component.
    pub(crate) transform: Transform,
    /// The global transform location in a space for a component.
    pub(crate) global_transform: GlobalTransform,
    /// If a layer has been modified, all are set here.
    pub(crate) modified_layer: ModifiedLayer,
}

/// A component bundle for `Tilemap` entities.
#[derive(Debug, Bundle)]
pub struct TilemapBundle {
    /// A `Tilemap` which maintains chunks and its tiles.
    pub tilemap: Tilemap,
    /// The transform location in a space for a component.
    pub transform: Transform,
    /// The global transform location in a space for a component.
    pub global_transform: GlobalTransform,
}
