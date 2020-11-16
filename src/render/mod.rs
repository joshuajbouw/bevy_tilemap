use crate::{chunk::ChunkDimensions, lib::*};

pub const CHUNK_PIPELINE_HANDLE: Handle<PipelineDescriptor> =
    Handle::weak_from_u64(PipelineDescriptor::TYPE_UUID, 2110840099625352487);

pub fn build_chunk_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    PipelineDescriptor {
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
        color_states: vec![ColorStateDescriptor {
            format: TextureFormat::default(),
            color_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendDescriptor {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: StencilStateDescriptor {
                front: StencilStateFaceDescriptor::IGNORE,
                back: StencilStateFaceDescriptor::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
        }),
        ..PipelineDescriptor::new(ShaderStages {
            vertex: shaders.add(Shader::from_glsl(
                ShaderStage::Vertex,
                include_str!("tile_map.vert"),
            )),
            fragment: Some(shaders.add(Shader::from_glsl(
                ShaderStage::Fragment,
                include_str!("tile_map.frag"),
            ))),
        })
    }
}

pub mod node {
    pub const CHUNK_DIMENSIONS: &str = "chunk_dimensions";
}

pub trait TilemapRenderGraphBuilder {
    fn add_tilemap_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl TilemapRenderGraphBuilder for RenderGraph {
    fn add_tilemap_graph(&mut self, resources: &Resources) -> &mut Self {
        self.add_system_node(
            node::CHUNK_DIMENSIONS,
            RenderResourcesNode::<ChunkDimensions>::new(false),
        );
        let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();
        let mut shaders = resources.get_mut::<Assets<Shader>>().unwrap();
        pipelines.set_untracked(CHUNK_PIPELINE_HANDLE, build_chunk_pipeline(&mut shaders));
        self
    }
}
