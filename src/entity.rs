use crate::{lib::*, render::CHUNK_PIPELINE_HANDLE};
use bevy::render::pipeline::PipelineSpecialization;

#[derive(Bundle)]
pub struct ChunkComponents {
    pub texture_atlas: Handle<TextureAtlas>,
    pub draw: Draw,
    pub render_pipelines: RenderPipelines,
    pub main_pass: MainPass,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
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
            texture_atlas: Default::default(),
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

impl ChunkComponents {
    pub fn new() -> ChunkComponents {
        ChunkComponents::default()
    }

    // pub fn set_material(&mut self, material: Handle<ColorMaterial>) {
    //     self.material = material;
    // }

    pub fn set_texture_atlas(&mut self, texture_atlas: Handle<TextureAtlas>) {
        self.texture_atlas = texture_atlas;
    }

    pub fn set_draw(&mut self, draw: Draw) {
        self.draw = draw;
    }

    pub fn set_mesh(&mut self, mesh: Handle<Mesh>) {
        self.mesh = mesh;
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn set_global_transform(&mut self, global_transform: GlobalTransform) {
        self.global_transform = global_transform;
    }

    pub fn add_render_pipeline(&mut self, pipeline: RenderPipeline) {
        self.render_pipelines.pipelines.push(pipeline);
    }
}
