use crate::{chunk::Chunk, dimension::Dimension2, lib::*, render::CHUNK_PIPELINE_HANDLE, Tilemap};
use bevy::render::pipeline::PipelineSpecialization;

/// A component that stores the dimensions of the Chunk for the renderer.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Default, RenderResources, RenderResource)]
#[render_resources(from_self)]
pub(crate) struct ChunkDimensions {
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

// /// A component that is used as a flag for dirty chunks that need updating.
pub(crate) struct DirtyLayer(pub(crate) usize);

/// A component bundle for `Chunk` entities.
#[derive(Bundle)]
pub(crate) struct ChunkComponents {
    pub(crate) chunk: Handle<Chunk>,
    /// The handle for a TextureAtlas which contains multiple textures.
    pub(crate) texture_atlas: Handle<TextureAtlas>,
    /// The chunk's dimensions which are passed to the renderer.
    pub(crate) chunk_dimensions: ChunkDimensions,
    /// A component that indicates how to draw a component.
    pub(crate) draw: Draw,
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
}

impl Default for ChunkComponents {
    fn default() -> ChunkComponents {
        let pipeline = RenderPipeline::specialized(
            CHUNK_PIPELINE_HANDLE,
            PipelineSpecialization {
                dynamic_bindings: vec![
                    // Transform
                    DynamicBinding {
                        bind_group: 2,
                        binding: 0,
                    },
                    // Chunk
                    DynamicBinding {
                        bind_group: 2,
                        binding: 1,
                    },
                ],
                ..Default::default()
            },
        );
        ChunkComponents {
            chunk: Default::default(),
            texture_atlas: Default::default(),
            chunk_dimensions: Default::default(),
            mesh: Default::default(),
            transform: Default::default(),
            render_pipelines: RenderPipelines::from_pipelines(vec![pipeline]),
            draw: Draw {
                is_transparent: true,
                ..Default::default()
            },
            main_pass: MainPass,
            global_transform: Default::default(),
        }
    }
}

/// A component bundle for `Tilemap` entities.
#[derive(Debug, Bundle)]
pub struct TilemapComponents {
    /// A `Tilemap` which maintains chunks and its tiles.
    pub tilemap: Tilemap,
    /// The transform location in a space for a component.
    pub transform: Transform,
    /// The global transform location in a space for a component.
    pub global_transform: GlobalTransform,
}
