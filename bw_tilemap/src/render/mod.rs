use bevy::{
    ecs::Resources,
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{
            BlendDescriptor, BlendFactor, BlendOperation, ColorStateDescriptor, ColorWrite,
            CompareFunction, CullMode, DepthStencilStateDescriptor, FrontFace, PipelineDescriptor,
            RasterizationStateDescriptor, StencilStateDescriptor, StencilStateFaceDescriptor,
        },
        render_graph::{AssetRenderResourcesNode, RenderGraph},
        shader::{ShaderStage, ShaderStages},
        texture::TextureFormat,
    },
};
use bw_bevy_assets::TileAtlas;

pub const TILEMAP_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 10430895678970401393);

pub fn build_tilemap_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    PipelineDescriptor {
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
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
        ..PipelineDescriptor::new(ShaderStages {
            vertex: shaders.add(Shader::from_glsl(
                ShaderStage::Vertex,
                include_str!("tilemap.vert"),
            )),
            fragment: Some(shaders.add(Shader::from_glsl(
                ShaderStage::Fragment,
                include_str!("tilemap.frag"),
            ))),
        })
    }
}

pub mod node {
    pub const TILEMAP: &str = "tilemap";
}

pub trait TilemapRenderGraphBuilder {
    fn add_tilemap_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl TilemapRenderGraphBuilder for RenderGraph {
    fn add_tilemap_graph(&mut self, resources: &Resources) -> &mut Self {
        self.add_system_node(
            node::TILEMAP,
            AssetRenderResourcesNode::<TileAtlas>::new(false),
        );

        let mut pipelines = resources
            .get_mut::<Assets<PipelineDescriptor>>()
            .expect("`PipelineDescriptor` is missing.");
        let mut shaders = resources
            .get_mut::<Assets<Shader>>()
            .expect("`Shader` is missing.");

        pipelines.set_untracked(
            TILEMAP_PIPELINE_HANDLE,
            build_tilemap_pipeline(&mut shaders),
        );

        self
    }
}

mod private {
    use super::RenderGraph;

    /// Seals the type.
    pub trait Sealed {}

    impl Sealed for RenderGraph {}
}
