use crevice::std140::AsStd140;
use wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    BufferSize, ColorTargetState, ColorWrites, FrontFace, MultisampleState, PolygonMode,
    PrimitiveState, PrimitiveTopology, SamplerBindingType, ShaderStages, TextureSampleType,
    TextureViewDimension, VertexFormat, VertexStepMode,
};

use crate::{
    asset::{Handle, HandleId},
    ecs::{FromWorld, Resource, World},
    render::{
        render_resource::{
            BindGroupLayout, FragmentState, RenderPipelineDescriptor, Shader, SpecializedPipeline,
            VertexBufferLayout, VertexState,
        },
        renderer::RenderDevice,
        texture::TEXTURE_FORMAT,
        view::ViewUniform,
    },
};

use super::UI_SHADER_HANDLE;

#[derive(Resource)]
pub struct UiPipeline {
    pub view_layout: BindGroupLayout,
    pub image_layout: BindGroupLayout,
}

impl FromWorld for UiPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(ViewUniform::std140_size_static() as u64),
                },
                count: None,
            }],
            label: Some("ui_view_layout"),
        });

        let image_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("ui_image_layout"),
        });

        UiPipeline {
            view_layout,
            image_layout,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct UiPipelineKey {}

impl SpecializedPipeline for UiPipeline {
    type Key = UiPipelineKey;
    /// FIXME: there are no specialization for now, should this be removed?
    fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
        let vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Vertex,
            vec![
                // position
                VertexFormat::Float32x3,
                // uv
                VertexFormat::Float32x2,
                // color
                VertexFormat::Float32x4,
            ],
        );

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: Handle::weak(HandleId::with_id::<Shader>(UI_SHADER_HANDLE)),
                entry_point: "vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: Handle::weak(HandleId::with_id::<Shader>(UI_SHADER_HANDLE)),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: Some(vec![self.view_layout.clone(), self.image_layout.clone()]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("ui_pipeline".into()),
        }
    }
}
