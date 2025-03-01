use std::sync::Arc;
use wgpu::{RenderPipelineDescriptor as RawRenderPipelineDescriptor, util::DeviceExt};

use crate::{
    ecs::Resource,
    render::render_resource::{
        BindGroup, BindGroupLayout, Buffer, ComputePipeline, RenderPipeline, Sampler, Texture,
    },
};

#[derive(Clone, Resource)]
pub struct RenderDevice {
    device: Arc<wgpu::Device>,
}

impl From<Arc<wgpu::Device>> for RenderDevice {
    fn from(device: Arc<wgpu::Device>) -> Self {
        Self { device }
    }
}

impl RenderDevice {
    #[inline]
    pub fn features(&self) -> wgpu::Features {
        self.device.features()
    }

    #[inline]
    pub fn limits(&self) -> wgpu::Limits {
        self.device.limits()
    }

    #[inline]
    pub fn create_shader_module(&self, desc: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        self.device.create_shader_module(desc)
    }

    #[inline]
    pub fn create_command_encoder(
        &self,
        desc: &wgpu::CommandEncoderDescriptor,
    ) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(desc)
    }

    #[inline]
    pub fn create_render_bundle_encoder(
        &self,
        desc: &wgpu::RenderBundleEncoderDescriptor,
    ) -> wgpu::RenderBundleEncoder {
        self.device.create_render_bundle_encoder(desc)
    }

    #[inline]
    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> BindGroup {
        let wgpu_bind_group = self.device.create_bind_group(desc);
        BindGroup::from(wgpu_bind_group)
    }

    #[inline]
    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> BindGroupLayout {
        BindGroupLayout::from(self.device.create_bind_group_layout(desc))
    }

    #[inline]
    pub fn create_pipeline_layout(
        &self,
        desc: &wgpu::PipelineLayoutDescriptor,
    ) -> wgpu::PipelineLayout {
        self.device.create_pipeline_layout(desc)
    }

    #[inline]
    pub fn create_render_pipeline(&self, desc: &RawRenderPipelineDescriptor) -> RenderPipeline {
        let wgpu_render_pipeline = self.device.create_render_pipeline(desc);
        RenderPipeline::from(wgpu_render_pipeline)
    }

    #[inline]
    pub fn create_compute_pipeline(
        &self,
        desc: &wgpu::ComputePipelineDescriptor,
    ) -> ComputePipeline {
        let wgpu_compute_pipeline = self.device.create_compute_pipeline(desc);
        ComputePipeline::from(wgpu_compute_pipeline)
    }

    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> Buffer {
        let wgpu_buffer = self.device.create_buffer(desc);
        Buffer::from(wgpu_buffer)
    }

    pub fn create_buffer_with_data(&self, desc: &wgpu::util::BufferInitDescriptor) -> Buffer {
        let wgpu_buffer = self.device.create_buffer_init(desc);
        Buffer::from(wgpu_buffer)
    }

    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> Texture {
        let wgpu_texture = self.device.create_texture(desc);
        Texture::from(wgpu_texture)
    }

    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> Sampler {
        let wgpu_sampler = self.device.create_sampler(desc);
        Sampler::from(wgpu_sampler)
    }

    pub fn configure_surface(&self, surface: &wgpu::Surface, config: &wgpu::SurfaceConfiguration) {
        surface.configure(&self.device, config)
    }

    pub fn wgpu_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn align_copy_bytes_per_row(row_bytes: usize) -> usize {
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - row_bytes % align) % align;
        row_bytes + padded_bytes_per_row_padding
    }
}
