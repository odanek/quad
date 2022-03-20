use std::{borrow::Cow, ops::Deref, sync::Arc};
use uuid::Uuid;
use wgpu::{
    BufferAddress, ColorTargetState, DepthStencilState, MultisampleState, PrimitiveState,
    VertexAttribute, VertexStepMode, VertexFormat,
};

use crate::asset::Handle;

use super::{BindGroupLayout, Shader};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct RenderPipelineId(Uuid);

#[derive(Clone, Debug)]
pub struct RenderPipeline {
    id: RenderPipelineId,
    value: Arc<wgpu::RenderPipeline>,
}

impl RenderPipeline {
    #[inline]
    pub fn id(&self) -> RenderPipelineId {
        self.id
    }
}

impl From<wgpu::RenderPipeline> for RenderPipeline {
    fn from(value: wgpu::RenderPipeline) -> Self {
        RenderPipeline {
            id: RenderPipelineId(Uuid::new_v4()),
            value: Arc::new(value),
        }
    }
}

impl Deref for RenderPipeline {
    type Target = wgpu::RenderPipeline;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ComputePipelineId(Uuid);

#[derive(Clone, Debug)]
pub struct ComputePipeline {
    id: ComputePipelineId,
    value: Arc<wgpu::ComputePipeline>,
}

impl ComputePipeline {
    #[inline]
    pub fn id(&self) -> ComputePipelineId {
        self.id
    }
}

impl From<wgpu::ComputePipeline> for ComputePipeline {
    fn from(value: wgpu::ComputePipeline) -> Self {
        ComputePipeline {
            id: ComputePipelineId(Uuid::new_v4()),
            value: Arc::new(value),
        }
    }
}

impl Deref for ComputePipeline {
    type Target = wgpu::ComputePipeline;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct RenderPipelineDescriptor {
    pub label: Option<Cow<'static, str>>,
    pub layout: Option<Vec<BindGroupLayout>>,
    pub vertex: VertexState,
    pub primitive: PrimitiveState,
    pub depth_stencil: Option<DepthStencilState>,
    pub multisample: MultisampleState,
    pub fragment: Option<FragmentState>,
}

#[derive(Clone, Debug)]
pub struct VertexState {
    pub shader: Handle<Shader>,
    pub shader_defs: Vec<String>,
    pub entry_point: Cow<'static, str>,
    pub buffers: Vec<VertexBufferLayout>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct VertexBufferLayout {
    pub array_stride: BufferAddress,
    pub step_mode: VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

impl VertexBufferLayout {
    /// Creates a new densely packed [`VertexBufferLayout`] from an iterator of vertex formats.
    /// Iteration order determines the `shader_location` and `offset` of the [`VertexAttributes`](VertexAttribute).
    /// The first iterated item will have a `shader_location` and `offset` of zero.
    /// The `array_stride` is the sum of the size of the iterated [`VertexFormats`](VertexFormat) (in bytes).
    pub fn from_vertex_formats<T: IntoIterator<Item = VertexFormat>>(
        step_mode: VertexStepMode,
        vertex_formats: T,
    ) -> Self {
        let mut offset = 0;
        let mut attributes = Vec::new();
        for (shader_location, format) in vertex_formats.into_iter().enumerate() {
            attributes.push(VertexAttribute {
                format,
                offset,
                shader_location: shader_location as u32,
            });
            offset += format.size();
        }

        VertexBufferLayout {
            array_stride: offset,
            step_mode,
            attributes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FragmentState {
    pub shader: Handle<Shader>,
    pub shader_defs: Vec<String>,
    pub entry_point: Cow<'static, str>,
    pub targets: Vec<ColorTargetState>,
}
