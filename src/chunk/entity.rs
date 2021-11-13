use crate::lib::*;

/// A component that is used as a flag for dirty chunks that need updating.
#[derive(Component, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub(crate) struct Modified(pub usize);

/// A component bundle for `Chunk` entities.
#[derive(Bundle)]
pub(crate) struct ChunkBundle {
    /// The point of the chunk.
    pub point: Point2,
    /// The handle for a TextureAtlas which contains multiple textures.
    pub texture_atlas: Handle<TextureAtlas>,
    /// A component that indicates how to draw a component.
    pub draw: Draw,
    /// A component that indicates if the component is visible.
    pub visible: Visible,
    /// The pipeline for the renderer.
    pub render_pipelines: RenderPipelines,
    /// A component that indicates that an entity should be drawn in the
    /// "main pass"
    pub main_pass: MainPass,
    /// A mesh of vertices for a component.
    pub mesh: Handle<Mesh>,
    /// The transform location in a space for a component.
    pub transform: Transform,
    /// The global transform location in a space for a component.
    pub global_transform: GlobalTransform,
    /// If a layer has been modified, all are set here.
    pub modified: Modified,
}
