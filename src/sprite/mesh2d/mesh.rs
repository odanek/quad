use cgm::{Matrix, SquareMatrix};
use crevice::std140::AsStd140;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, BufferBindingType, BufferSize, ColorTargetState, ColorWrites,
    Extent3d, Face, FrontFace, ImageCopyTexture, ImageDataLayout, MultisampleState, Origin3d,
    PolygonMode, PrimitiveState, PrimitiveTopology, ShaderStages, TextureAspect, TextureDimension,
    TextureFormat, TextureViewDescriptor,
};

use crate::{
    app::{App, Stage},
    asset::{load_internal_asset, Handle, HandleUntyped},
    ecs::{
        Commands, Component, Entity, FromWorld, IntoSystem, Local, Query, Res, Resource,
        SystemParamItem, With, World,
    },
    reflect::TypeUuid,
    render::{
        mesh::{GpuBufferInfo, Mesh, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_component::{uniform_component_plugin, ComponentUniforms, DynamicUniformIndex},
        render_phase::{EntityRenderCommand, RenderCommandResult, TrackedRenderPass},
        render_resource::{
            BindGroup, BindGroupLayout, FragmentState, RenderPipelineDescriptor, Sampler, Shader,
            SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines,
            TextureView, VertexState,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage, Image, TextureFormatPixelInfo},
        view::{ComputedVisibility, ExtractedView, ViewUniform, ViewUniformOffset, ViewUniforms},
    },
    transform::GlobalTransform,
    ty::{Mat4, Size},
};

/// Component for rendering with meshes in the 2d pipeline, usually with a [2d material](crate::Material2d) such as [`ColorMaterial`](crate::ColorMaterial).
///
/// It wraps a [`Handle<Mesh>`] to differentiate from the 3d pipelines which use the handles directly as components
#[derive(Default, Clone, Component)]
pub struct Mesh2dHandle(pub Handle<Mesh>);

impl From<Handle<Mesh>> for Mesh2dHandle {
    fn from(handle: Handle<Mesh>) -> Self {
        Self(handle)
    }
}

pub const MESH2D_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2971387252468633715);

pub fn mesh_2d_render_plugin(app: &mut App, render_app: &mut App) {
    load_internal_asset!(app, MESH2D_SHADER_HANDLE, "mesh2d.wgsl", Shader::from_wgsl);

    uniform_component_plugin::<Mesh2dUniform>(render_app);

    render_app
        .init_resource::<Mesh2dPipeline>()
        .init_resource::<SpecializedMeshPipelines<Mesh2dPipeline>>()
        .add_system_to_stage(Stage::RenderExtract, extract_mesh2d.system(&mut app.world))
        .add_system_to_stage(Stage::RenderQueue, &queue_mesh2d_bind_group)
        .add_system_to_stage(Stage::RenderQueue, &queue_mesh2d_view_bind_groups);
}

#[derive(Component, AsStd140, Clone)]
pub struct Mesh2dUniform {
    pub transform: Mat4,
    pub inverse_transpose_model: Mat4,
    pub flags: u32,
}

// NOTE: These must match the bit flags in bevy_sprite/src/mesh2d/mesh2d.wgsl!
bitflags::bitflags! {
    #[repr(transparent)]
    struct MeshFlags: u32 {
        const NONE                       = 0;
        const UNINITIALIZED              = 0xFFFF;
    }
}

pub fn extract_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    query: Query<(Entity, &ComputedVisibility, &GlobalTransform, &Mesh2dHandle)>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility, transform, handle) in query.iter() {
        if !computed_visibility.is_visible {
            continue;
        }
        let transform = transform.compute_matrix();
        values.push((
            entity,
            (
                Mesh2dHandle(handle.0.clone_weak()),
                Mesh2dUniform {
                    flags: MeshFlags::empty().bits,
                    transform,
                    inverse_transpose_model: transform.inverse().unwrap().transpose(),
                },
            ),
        ));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}

#[derive(Clone, Resource)]
pub struct Mesh2dPipeline {
    pub view_layout: BindGroupLayout,
    pub mesh_layout: BindGroupLayout,
    // This dummy white texture is to be used in place of optional textures
    pub dummy_white_gpu_image: GpuImage,
}

impl FromWorld for Mesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // View
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(ViewUniform::std140_size_static() as u64),
                    },
                    count: None,
                },
            ],
            label: Some("mesh2d_view_layout"),
        });

        let mesh_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(Mesh2dUniform::std140_size_static() as u64),
                },
                count: None,
            }],
            label: Some("mesh2d_layout"),
        });
        // A 1x1x1 'all 1.0' texture to use as a dummy texture to use in place of optional StandardMaterial textures
        let dummy_white_gpu_image = {
            let image = Image::new_fill(
                Extent3d::default(),
                TextureDimension::D2,
                &[255u8; 4],
                TextureFormat::bevy_default(),
            );
            let texture = render_device.create_texture(&image.texture_descriptor);
            let sampler = render_device.create_sampler(&image.sampler_descriptor);

            let format_size = image.texture_descriptor.format.pixel_size();
            let render_queue = world.resource_mut::<RenderQueue>();
            render_queue.write_texture(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                &image.data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZeroU32::new(
                            image.texture_descriptor.size.width * format_size as u32,
                        )
                        .unwrap(),
                    ),
                    rows_per_image: None,
                },
                image.texture_descriptor.size,
            );

            let texture_view = texture.create_view(&TextureViewDescriptor::default());
            GpuImage {
                texture,
                texture_view,
                texture_format: image.texture_descriptor.format,
                sampler,
                size: Size::new(
                    image.texture_descriptor.size.width as f32,
                    image.texture_descriptor.size.height as f32,
                ),
            }
        };
        Mesh2dPipeline {
            view_layout,
            mesh_layout,
            dummy_white_gpu_image,
        }
    }
}

impl Mesh2dPipeline {
    pub fn get_image_texture<'a>(
        &'a self,
        gpu_images: &'a RenderAssets<Image>,
        handle_option: &Option<Handle<Image>>,
    ) -> Option<(&'a TextureView, &'a Sampler)> {
        if let Some(handle) = handle_option {
            let gpu_image = gpu_images.get(handle)?;
            Some((&gpu_image.texture_view, &gpu_image.sampler))
        } else {
            Some((
                &self.dummy_white_gpu_image.texture_view,
                &self.dummy_white_gpu_image.sampler,
            ))
        }
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    // NOTE: Apparently quadro drivers support up to 64x MSAA.
    // MSAA uses the highest 6 bits for the MSAA sample count - 1 to support up to 64x MSAA.
    // FIXME: make normals optional?
    pub struct Mesh2dPipelineKey: u32 {
        const NONE                        = 0;
        const MSAA_RESERVED_BITS          = Mesh2dPipelineKey::MSAA_MASK_BITS << Mesh2dPipelineKey::MSAA_SHIFT_BITS;
        const PRIMITIVE_TOPOLOGY_RESERVED_BITS = Mesh2dPipelineKey::PRIMITIVE_TOPOLOGY_MASK_BITS << Mesh2dPipelineKey::PRIMITIVE_TOPOLOGY_SHIFT_BITS;
    }
}

impl Mesh2dPipelineKey {
    const MSAA_MASK_BITS: u32 = 0b111111;
    const MSAA_SHIFT_BITS: u32 = 32 - 6;
    const PRIMITIVE_TOPOLOGY_MASK_BITS: u32 = 0b111;
    const PRIMITIVE_TOPOLOGY_SHIFT_BITS: u32 = Self::MSAA_SHIFT_BITS - 3;

    pub fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits = ((msaa_samples - 1) & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS;
        Mesh2dPipelineKey::from_bits(msaa_bits).unwrap()
    }

    pub fn msaa_samples(&self) -> u32 {
        ((self.bits >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS) + 1
    }

    pub fn from_primitive_topology(primitive_topology: PrimitiveTopology) -> Self {
        let primitive_topology_bits = ((primitive_topology as u32)
            & Self::PRIMITIVE_TOPOLOGY_MASK_BITS)
            << Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS;
        Mesh2dPipelineKey::from_bits(primitive_topology_bits).unwrap()
    }

    pub fn primitive_topology(&self) -> PrimitiveTopology {
        let primitive_topology_bits =
            (self.bits >> Self::PRIMITIVE_TOPOLOGY_SHIFT_BITS) & Self::PRIMITIVE_TOPOLOGY_MASK_BITS;
        match primitive_topology_bits {
            x if x == PrimitiveTopology::PointList as u32 => PrimitiveTopology::PointList,
            x if x == PrimitiveTopology::LineList as u32 => PrimitiveTopology::LineList,
            x if x == PrimitiveTopology::LineStrip as u32 => PrimitiveTopology::LineStrip,
            x if x == PrimitiveTopology::TriangleList as u32 => PrimitiveTopology::TriangleList,
            x if x == PrimitiveTopology::TriangleStrip as u32 => PrimitiveTopology::TriangleStrip,
            _ => PrimitiveTopology::default(),
        }
    }
}

impl SpecializedMeshPipeline for Mesh2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let vertex_attributes = vec![
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
        ];

        let shader_defs = Vec::new();

        let vertex_buffer_layout = layout.get_layout(&vertex_attributes)?;

        Ok(RenderPipelineDescriptor {
            vertex: VertexState {
                shader: MESH2D_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: shader_defs.clone(),
                buffers: vec![vertex_buffer_layout],
            },
            fragment: Some(FragmentState {
                shader: MESH2D_SHADER_HANDLE.typed::<Shader>(),
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(vec![self.view_layout.clone(), self.mesh_layout.clone()]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("transparent_mesh2d_pipeline".into()),
        })
    }
}

#[derive(Resource)]
pub struct Mesh2dBindGroup {
    pub value: BindGroup,
}

pub fn queue_mesh2d_bind_group(
    mut commands: Commands,
    mesh2d_pipeline: Res<Mesh2dPipeline>,
    render_device: Res<RenderDevice>,
    mesh2d_uniforms: Res<ComponentUniforms<Mesh2dUniform>>,
) {
    if let Some(binding) = mesh2d_uniforms.uniforms().binding() {
        commands.insert_resource(Mesh2dBindGroup {
            value: render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: binding,
                }],
                label: Some("mesh2d_bind_group"),
                layout: &mesh2d_pipeline.mesh_layout,
            }),
        });
    }
}

#[derive(Component)]
pub struct Mesh2dViewBindGroup {
    pub value: BindGroup,
}

pub fn queue_mesh2d_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mesh2d_pipeline: Res<Mesh2dPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in views.iter() {
            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
                label: Some("mesh2d_view_bind_group"),
                layout: &mesh2d_pipeline.view_layout,
            });

            commands.entity(entity).insert(Mesh2dViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}

pub struct SetMesh2dViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetMesh2dViewBindGroup<I> {
    type Param =
        Query<'static, 'static, (&'static ViewUniformOffset, &'static Mesh2dViewBindGroup)>;
    #[inline]
    fn render<'w>(
        view: Entity,
        _item: Entity,
        view_query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (view_uniform, mesh2d_view_bind_group) = view_query.get(view).unwrap();
        pass.set_bind_group(I, &mesh2d_view_bind_group.value, &[view_uniform.offset]);

        RenderCommandResult::Success
    }
}

pub struct SetMesh2dBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetMesh2dBindGroup<I> {
    type Param = (
        Res<'static, Mesh2dBindGroup>,
        Query<'static, 'static, &'static DynamicUniformIndex<Mesh2dUniform>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (mesh2d_bind_group, mesh2d_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh2d_index = mesh2d_query.get(item).unwrap();
        pass.set_bind_group(
            I,
            &mesh2d_bind_group.into_inner().value,
            &[mesh2d_index.index()],
        );
        RenderCommandResult::Success
    }
}

pub struct DrawMesh2d;
impl EntityRenderCommand for DrawMesh2d {
    type Param = (
        Res<'static, RenderAssets<Mesh>>,
        Query<'static, 'static, &'static Mesh2dHandle>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh2d_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = &mesh2d_query.get(item).unwrap().0;
        if let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) {
            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                GpuBufferInfo::NonIndexed { vertex_count } => {
                    pass.draw(0..*vertex_count, 0..1);
                }
            }
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}