use crate::lib::*;

macro_rules! build_chunk_pipeline {
    ($handle: ident, $id: expr, $name: ident, $file: expr) => {
        /// The constant render pipeline for a chunk.
        pub(crate) const $handle: Handle<PipelineDescriptor> =
            Handle::weak_from_u64(PipelineDescriptor::TYPE_UUID, $id);

        /// Builds the chunk render pipeline.
        fn $name(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
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
                    vertex: shaders
                        .add(Shader::from_glsl(ShaderStage::Vertex, include_str!($file))),
                    fragment: Some(shaders.add(Shader::from_glsl(
                        ShaderStage::Fragment,
                        include_str!("tilemap.frag"),
                    ))),
                })
            }
        }
    };
}

build_chunk_pipeline!(
    CHUNK_SQUARE_PIPELINE,
    2110840099625352487,
    build_chunk_square_pipeline,
    "tilemap-square.vert"
);
build_chunk_pipeline!(
    CHUNK_HEX_X_PIPELINE,
    7038597873061171051,
    build_chunk_hex_x,
    "tilemap-hex-x.vert"
);
build_chunk_pipeline!(
    CHUNK_HEX_Y_PIPELINE,
    4304966217182648108,
    build_chunk_hex_y,
    "tilemap-hex-y.vert"
);
build_chunk_pipeline!(
    CHUNK_HEXCOLS_EVEN_PIPELINE,
    7604280309043018950,
    build_chunk_hexcols_even,
    "tilemap-hexcols-even.vert"
);
build_chunk_pipeline!(
    CHUNK_HEXCOLS_ODD_PIPELINE,
    3111565682159860869,
    build_chunk_hexcols_odd,
    "tilemap-hexcols-odd.vert"
);
build_chunk_pipeline!(
    CHUNK_HEXROWS_EVEN_PIPELINE,
    1670470246078408352,
    build_chunk_hexrows_even,
    "tilemap-hexrows-even.vert"
);
build_chunk_pipeline!(
    CHUNK_HEXROWS_ODD_PIPELINE,
    8160067835497533408,
    build_chunk_hexrows_odd,
    "tilemap-hexrows-odd.vert"
);

/// Topology of the tilemap grid (square or hex)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridTopology {
    /// Square grid
    Square,
    /// Hex grid with rows offset (hexes with pointy top).
    HexY,
    /// Hex grid with columns offset (hexes with flat top).
    HexX,
    /// Hex grid with offset on even rows (hexes with pointy top).
    HexEvenRows,
    /// Hex grid with offset on odd rows (hexes with pointy top).
    HexOddRows,
    /// Hex grid with offset on even columns (hexes with flat top).
    HexEvenCols,
    /// Hex grid with offset on odd columns (hexes with flat top).
    HexOddCols,
}

impl GridTopology {
    /// Takes a grid topology and returns a handle.
    pub(crate) fn to_pipeline_handle(&self) -> Handle<PipelineDescriptor> {
        use GridTopology::*;
        match self {
            Square => CHUNK_SQUARE_PIPELINE,
            HexY => CHUNK_HEX_Y_PIPELINE,
            HexX => CHUNK_HEX_X_PIPELINE,
            HexEvenRows => CHUNK_HEXROWS_EVEN_PIPELINE,
            HexOddRows => CHUNK_HEXROWS_ODD_PIPELINE,
            HexEvenCols => CHUNK_HEXCOLS_EVEN_PIPELINE,
            HexOddCols => CHUNK_HEXCOLS_ODD_PIPELINE,
        }
    }
}

/// A trait which implements the tilemap graph to a render graph.
pub trait TilemapRenderGraphBuilder {
    /// Adds the tilemaps render graph.
    fn add_tilemap_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl TilemapRenderGraphBuilder for RenderGraph {
    fn add_tilemap_graph(&mut self, resources: &Resources) -> &mut Self {
        let mut pipelines = resources
            .get_mut::<Assets<PipelineDescriptor>>()
            .expect("`PipelineDescriptor` is missing.");
        let mut shaders = resources
            .get_mut::<Assets<Shader>>()
            .expect("`Shader` is missing.");

        pipelines.set_untracked(
            CHUNK_SQUARE_PIPELINE,
            build_chunk_square_pipeline(&mut shaders),
        );
        pipelines.set_untracked(CHUNK_HEX_X_PIPELINE, build_chunk_hex_x(&mut shaders));
        pipelines.set_untracked(CHUNK_HEX_Y_PIPELINE, build_chunk_hex_y(&mut shaders));
        pipelines.set_untracked(
            CHUNK_HEXCOLS_EVEN_PIPELINE,
            build_chunk_hexcols_even(&mut shaders),
        );
        pipelines.set_untracked(
            CHUNK_HEXCOLS_ODD_PIPELINE,
            build_chunk_hexcols_odd(&mut shaders),
        );
        pipelines.set_untracked(
            CHUNK_HEXROWS_EVEN_PIPELINE,
            build_chunk_hexrows_even(&mut shaders),
        );
        pipelines.set_untracked(
            CHUNK_HEXROWS_ODD_PIPELINE,
            build_chunk_hexrows_odd(&mut shaders),
        );

        self
    }
}

/// Prevents the traits in this module from being implemented outside the crate.
mod private {
    use super::RenderGraph;

    /// Seals the type.
    pub trait Sealed {}

    impl Sealed for RenderGraph {}
}
