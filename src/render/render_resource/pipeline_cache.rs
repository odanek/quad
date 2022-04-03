use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};
use thiserror::Error;
use wgpu::{
    FragmentState as RawFragmentState, PipelineLayoutDescriptor,
    RenderPipelineDescriptor as RawRenderPipelineDescriptor, ShaderModule, VertexBufferLayout,
    VertexState as RawVertexState,
};

use crate::{
    asset::{AssetEvent, Assets, Handle},
    ecs::{EventReader, Res, ResMut, Resource},
    render::{renderer::RenderDevice, RenderWorld},
};

use super::{BindGroupLayout, BindGroupLayoutId, RenderPipeline, RenderPipelineDescriptor, Shader};

#[derive(Default)]
pub struct ShaderData {
    pipelines: HashSet<CachedPipelineId>,
    processed_shader: Option<Arc<ShaderModule>>,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct CachedPipelineId(usize);

impl CachedPipelineId {
    pub const INVALID: Self = CachedPipelineId(usize::MAX);
}

#[derive(Default)]
struct ShaderCache {
    data: HashMap<Handle<Shader>, ShaderData>,
    shaders: HashMap<Handle<Shader>, Shader>,
}

impl ShaderCache {
    fn get(
        &mut self,
        render_device: &RenderDevice,
        pipeline: CachedPipelineId,
        handle: &Handle<Shader>,
    ) -> Result<Arc<ShaderModule>, RenderPipelineError> {
        let shader = self
            .shaders
            .get(handle)
            .ok_or_else(|| RenderPipelineError::ShaderNotLoaded(handle.clone_weak()))?;

        let data = self.data.entry(handle.clone_weak()).or_default();

        let module = data.processed_shader.get_or_insert_with(|| {
            let module_descriptor = shader.get_module_descriptor();
            Arc::new(render_device.create_shader_module(&module_descriptor))
        });

        data.pipelines.insert(pipeline);

        Ok(module.clone())
    }

    fn clear(&mut self, handle: &Handle<Shader>) -> Vec<CachedPipelineId> {
        if let Some(data) = self.data.get_mut(handle) {
            data.processed_shader = None;
            data.pipelines.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    fn set_shader(&mut self, handle: &Handle<Shader>, shader: Shader) -> Vec<CachedPipelineId> {
        let pipelines_to_queue = self.clear(handle);
        self.shaders.insert(handle.clone_weak(), shader);
        pipelines_to_queue
    }

    fn remove(&mut self, handle: &Handle<Shader>) -> Vec<CachedPipelineId> {
        let pipelines_to_queue = self.clear(handle);
        self.shaders.remove(handle);
        pipelines_to_queue
    }
}

#[derive(Default)]
struct LayoutCache {
    layouts: HashMap<Vec<BindGroupLayoutId>, wgpu::PipelineLayout>,
}

impl LayoutCache {
    fn get(
        &mut self,
        render_device: &RenderDevice,
        bind_group_layouts: &[BindGroupLayout],
    ) -> &wgpu::PipelineLayout {
        let key = bind_group_layouts.iter().map(|l| l.id()).collect();
        self.layouts.entry(key).or_insert_with(|| {
            let bind_group_layouts = bind_group_layouts
                .iter()
                .map(|l| l.value())
                .collect::<Vec<_>>();
            render_device.create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: &bind_group_layouts,
                ..Default::default()
            })
        })
    }
}

#[derive(Resource)]
pub struct RenderPipelineCache {
    layout_cache: LayoutCache,
    shader_cache: ShaderCache,
    device: RenderDevice,
    pipelines: Vec<CachedPipeline>,
    waiting_pipelines: HashSet<CachedPipelineId>,
}

struct CachedPipeline {
    descriptor: RenderPipelineDescriptor,
    state: CachedPipelineState,
}

#[derive(Debug)]
pub enum CachedPipelineState {
    Queued,
    Ok(RenderPipeline),
    Err(RenderPipelineError),
}

impl CachedPipelineState {
    pub fn unwrap(&self) -> &RenderPipeline {
        match self {
            CachedPipelineState::Ok(pipeline) => pipeline,
            CachedPipelineState::Queued => {
                panic!("Pipeline has not been compiled yet. It is still in the 'Queued' state.")
            }
            CachedPipelineState::Err(err) => panic!("{}", err),
        }
    }
}

#[derive(Error, Debug)]
pub enum RenderPipelineError {
    #[error(
        "Pipeline cound not be compiled because the following shader is not loaded yet: {0:?}"
    )]
    ShaderNotLoaded(Handle<Shader>),
}

impl RenderPipelineCache {
    pub fn new(device: RenderDevice) -> Self {
        Self {
            device,
            layout_cache: Default::default(),
            shader_cache: Default::default(),
            waiting_pipelines: Default::default(),
            pipelines: Default::default(),
        }
    }

    #[inline]
    pub fn get_state(&self, id: CachedPipelineId) -> &CachedPipelineState {
        &self.pipelines[id.0].state
    }

    #[inline]
    pub fn get_descriptor(&self, id: CachedPipelineId) -> &RenderPipelineDescriptor {
        &self.pipelines[id.0].descriptor
    }

    #[inline]
    pub fn get(&self, id: CachedPipelineId) -> Option<&RenderPipeline> {
        if let CachedPipelineState::Ok(pipeline) = &self.pipelines[id.0].state {
            Some(pipeline)
        } else {
            None
        }
    }

    pub fn queue(&mut self, descriptor: RenderPipelineDescriptor) -> CachedPipelineId {
        let id = CachedPipelineId(self.pipelines.len());
        self.pipelines.push(CachedPipeline {
            descriptor,
            state: CachedPipelineState::Queued,
        });
        self.waiting_pipelines.insert(id);
        id
    }

    fn set_shader(&mut self, handle: &Handle<Shader>, shader: &Shader) {
        let pipelines_to_queue = self.shader_cache.set_shader(handle, shader.clone());
        for cached_pipeline in pipelines_to_queue {
            self.pipelines[cached_pipeline.0].state = CachedPipelineState::Queued;
            self.waiting_pipelines.insert(cached_pipeline);
        }
    }

    fn remove_shader(&mut self, shader: &Handle<Shader>) {
        let pipelines_to_queue = self.shader_cache.remove(shader);
        for cached_pipeline in pipelines_to_queue {
            self.pipelines[cached_pipeline.0].state = CachedPipelineState::Queued;
            self.waiting_pipelines.insert(cached_pipeline);
        }
    }

    pub fn process_queue(&mut self) {
        let pipelines = std::mem::take(&mut self.waiting_pipelines);
        for id in pipelines {
            let state = &mut self.pipelines[id.0];
            match &state.state {
                CachedPipelineState::Ok(_) => continue,
                CachedPipelineState::Queued => {}
                CachedPipelineState::Err(err) => {
                    match err {
                        RenderPipelineError::ShaderNotLoaded(_) => { /* retry */ }
                    }
                }
            }

            let descriptor = &state.descriptor;
            let vertex_module =
                match self
                    .shader_cache
                    .get(&self.device, id, &descriptor.vertex.shader)
                {
                    Ok(module) => module,
                    Err(err) => {
                        state.state = CachedPipelineState::Err(err);
                        self.waiting_pipelines.insert(id);
                        continue;
                    }
                };

            let fragment_data = if let Some(fragment) = &descriptor.fragment {
                let fragment_module =
                    match self.shader_cache.get(&self.device, id, &fragment.shader) {
                        Ok(module) => module,
                        Err(err) => {
                            state.state = CachedPipelineState::Err(err);
                            self.waiting_pipelines.insert(id);
                            continue;
                        }
                    };
                Some((
                    fragment_module,
                    fragment.entry_point.deref(),
                    &fragment.targets,
                ))
            } else {
                None
            };

            let vertex_buffer_layouts = descriptor
                .vertex
                .buffers
                .iter()
                .map(|layout| VertexBufferLayout {
                    array_stride: layout.array_stride,
                    attributes: &layout.attributes,
                    step_mode: layout.step_mode,
                })
                .collect::<Vec<_>>();

            let layout = if let Some(layout) = &descriptor.layout {
                Some(self.layout_cache.get(&self.device, layout))
            } else {
                None
            };

            let descriptor = RawRenderPipelineDescriptor {
                multiview: None,
                depth_stencil: descriptor.depth_stencil.clone(),
                label: descriptor.label.as_deref(),
                layout,
                multisample: descriptor.multisample,
                primitive: descriptor.primitive,
                vertex: RawVertexState {
                    buffers: &vertex_buffer_layouts,
                    entry_point: descriptor.vertex.entry_point.deref(),
                    module: &vertex_module,
                },
                fragment: fragment_data
                    .as_ref()
                    .map(|(module, entry_point, targets)| RawFragmentState {
                        entry_point,
                        module,
                        targets,
                    }),
            };

            let pipeline = self.device.create_render_pipeline(&descriptor);
            state.state = CachedPipelineState::Ok(pipeline);
        }
    }

    pub(crate) fn process_pipeline_queue_system(mut cache: ResMut<Self>) {
        cache.process_queue();
    }

    pub(crate) fn extract_shaders(
        mut world: ResMut<RenderWorld>,
        shaders: Res<Assets<Shader>>,
        mut events: EventReader<AssetEvent<Shader>>,
    ) {
        let mut cache = world.get_resource_mut::<Self>().unwrap();
        for event in events.iter() {
            match event {
                AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                    if let Some(shader) = shaders.get(handle) {
                        cache.set_shader(handle, shader);
                    }
                }
                AssetEvent::Removed { handle } => cache.remove_shader(handle),
            }
        }
    }
}
