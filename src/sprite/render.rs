use std::{cmp::Ordering, collections::HashMap, ops::BitOr};

use bytemuck::{Pod, Zeroable};
use cgm::{ElementWise, Zero};
use crevice::std140::AsStd140;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, BufferBindingType, BufferSize, BufferUsages,
    ColorTargetState, ColorWrites, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
    PrimitiveTopology, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
    VertexFormat, VertexStepMode,
};

use crate::{
    asset::{AssetEvent, Assets, Handle, HandleId},
    ecs::{
        Commands, Component, Entity, EventReader, FromWorld, Query, Res, ResMut, Resource,
        SystemParamItem, World,
    },
    pipeline::Transparent2d,
    render::{
        color::Color,
        extract_param::Extract,
        render_asset::RenderAssets,
        render_phase::{
            BatchedPhaseItem, DrawFunctions, EntityRenderCommand, RenderCommand,
            RenderCommandResult, RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupLayout, BufferVec, FragmentState, RenderPipelineCache,
            RenderPipelineDescriptor, Shader, SpecializedPipeline, SpecializedPipelines,
            VertexBufferLayout, VertexState,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{Image, TEXTURE_FORMAT},
        view::{ComputedVisibility, Msaa, ViewUniform, ViewUniformOffset, ViewUniforms},
    },
    transform::GlobalTransform,
    ty::{FloatOrd, Vec2},
};

use super::{
    Rect, SPRITE_COLORED_SHADER_HANDLE, SPRITE_SHADER_HANDLE, Sprite, TextureAtlas,
    TextureAtlasSprite,
};

#[derive(Resource)]
pub struct SpritePipeline {
    view_layout: BindGroupLayout,
    material_layout: BindGroupLayout,
}

impl FromWorld for SpritePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

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
            label: Some("sprite_view_layout"),
        });

        let material_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            label: Some("sprite_material_layout"),
        });

        SpritePipeline {
            view_layout,
            material_layout,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpritePipelineKey(u32);

impl SpritePipelineKey {
    const COLORED: Self = SpritePipelineKey(1 << 0);
    const MSAA_MASK_BITS: u32 = 0b111111;
    const MSAA_SHIFT_BITS: u32 = 32 - Self::MSAA_MASK_BITS.count_ones();

    pub const fn contains(&self, key: Self) -> bool {
        self.0 & key.0 == key.0
    }

    pub const fn from_msaa_samples(msaa_samples: u32) -> Self {
        Self(((msaa_samples - 1) & Self::MSAA_MASK_BITS) << Self::MSAA_SHIFT_BITS)
    }

    pub const fn msaa_samples(&self) -> u32 {
        ((self.0 >> Self::MSAA_SHIFT_BITS) & Self::MSAA_MASK_BITS) + 1
    }
}

impl BitOr for SpritePipelineKey {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl SpecializedPipeline for SpritePipeline {
    type Key = SpritePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut formats = vec![
            // position
            VertexFormat::Float32x3,
            // uv
            VertexFormat::Float32x2,
        ];

        if key.contains(SpritePipelineKey::COLORED) {
            // color
            formats.push(VertexFormat::Uint32);
        }

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        let colored = key.contains(SpritePipelineKey::COLORED);
        let shader_handle = HandleId::with_id::<Shader>(if colored {
            SPRITE_COLORED_SHADER_HANDLE
        } else {
            SPRITE_SHADER_HANDLE
        });

        RenderPipelineDescriptor {
            vertex: VertexState {
                shader: Handle::weak(shader_handle),
                entry_point: "vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: Handle::weak(shader_handle),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: Some(vec![self.view_layout.clone(), self.material_layout.clone()]),
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
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("sprite_pipeline".into()),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ExtractedSprite {
    pub transform: GlobalTransform,
    pub color: Color,
    /// Select an area of the texture
    pub rect: Option<Rect>,
    /// Change the on-screen size of the sprite
    pub custom_size: Option<Vec2>,
    /// Handle to the `Image` of this sprite
    /// PERF: storing a `HandleId` instead of `Handle<Image>` enables some optimizations (`ExtractedSprite` becomes `Copy` and doesn't need to be dropped)
    pub image_handle_id: HandleId,
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor: Vec2,
}

#[derive(Default, Resource)]
pub struct ExtractedSprites {
    pub sprites: Vec<ExtractedSprite>,
}

#[derive(Default, Resource)]
pub struct SpriteAssetEvents {
    pub images: Vec<AssetEvent<Image>>,
}

pub fn extract_sprite_events(
    mut events: ResMut<SpriteAssetEvents>,
    mut image_events: Extract<EventReader<AssetEvent<Image>>>,
) {
    let SpriteAssetEvents { ref mut images } = *events;
    images.clear();

    for image in image_events.iter() {
        // AssetEvent: !Clone
        images.push(match image {
            AssetEvent::Created { handle } => AssetEvent::Created {
                handle: handle.clone_weak(),
            },
            AssetEvent::Modified { handle } => AssetEvent::Modified {
                handle: handle.clone_weak(),
            },
            AssetEvent::Removed { handle } => AssetEvent::Removed {
                handle: handle.clone_weak(),
            },
        });
    }
}

#[allow(clippy::type_complexity)]
pub fn extract_sprites(
    mut extracted_sprites: ResMut<ExtractedSprites>,
    texture_atlases: Extract<Res<Assets<TextureAtlas>>>,
    sprite_query: Extract<
        Query<(
            &ComputedVisibility,
            &Sprite,
            &GlobalTransform,
            &Handle<Image>,
        )>,
    >,
    atlas_query: Extract<
        Query<(
            &ComputedVisibility,
            &TextureAtlasSprite,
            &GlobalTransform,
            &Handle<TextureAtlas>,
        )>,
    >,
) {
    extracted_sprites.sprites.clear();
    for (visibility, sprite, transform, handle) in sprite_query.iter() {
        if !visibility.is_visible() {
            continue;
        }
        // PERF: we don't check in this function that the `Image` asset is ready, since it should be in most cases and hashing the handle is expensive
        extracted_sprites.sprites.push(ExtractedSprite {
            color: sprite.color,
            transform: *transform,
            // Use the full texture
            rect: sprite.rect,
            // Pass the custom size
            custom_size: sprite.custom_size,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
            image_handle_id: handle.id,
            anchor: sprite.anchor.as_vec(),
        });
    }
    for (visibility, atlas_sprite, transform, texture_atlas_handle) in atlas_query.iter() {
        if !visibility.is_visible() {
            continue;
        }
        if let Some(texture_atlas) = texture_atlases.get(texture_atlas_handle) {
            let rect = Some(texture_atlas.textures[atlas_sprite.index]);
            extracted_sprites.sprites.push(ExtractedSprite {
                color: atlas_sprite.color,
                transform: *transform,
                // Select the area in the texture atlas
                rect,
                // Pass the custom size
                custom_size: atlas_sprite.custom_size,
                flip_x: atlas_sprite.flip_x,
                flip_y: atlas_sprite.flip_y,
                image_handle_id: texture_atlas.texture.id,
                anchor: atlas_sprite.anchor.as_vec(),
            });
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SpriteVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ColoredSpriteVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: u32,
}

#[derive(Resource)]
pub struct SpriteMeta {
    vertices: BufferVec<SpriteVertex>,
    colored_vertices: BufferVec<ColoredSpriteVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for SpriteMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            colored_vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

const QUAD_INDICES: [usize; 6] = [0, 2, 3, 0, 1, 2];

const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0., 1.),
    Vec2::new(1., 1.),
    Vec2::new(1., 0.),
    Vec2::new(0., 0.),
];

#[derive(Component, Eq, PartialEq, Copy, Clone)]
pub struct SpriteBatch {
    image_handle_id: HandleId,
    colored: bool,
}

#[derive(Default, Resource)]
pub struct ImageBindGroups {
    values: HashMap<Handle<Image>, BindGroup>,
}

#[allow(clippy::too_many_arguments)]
pub fn queue_sprites(
    mut commands: Commands,
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut sprite_meta: ResMut<SpriteMeta>,
    view_uniforms: Res<ViewUniforms>,
    sprite_pipeline: Res<SpritePipeline>,
    mut pipelines: ResMut<SpecializedPipelines<SpritePipeline>>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    mut image_bind_groups: ResMut<ImageBindGroups>,
    gpu_images: Res<RenderAssets<Image>>,
    msaa: Res<Msaa>,
    mut extracted_sprites: ResMut<ExtractedSprites>,
    mut views: Query<&mut RenderPhase<Transparent2d>>,
    events: Res<SpriteAssetEvents>,
) {
    // If an image has changed, the GpuImage has (probably) changed
    for event in &events.images {
        match event {
            AssetEvent::Created { .. } => None,
            AssetEvent::Modified { handle } => image_bind_groups.values.remove(handle),
            AssetEvent::Removed { handle } => image_bind_groups.values.remove(handle),
        };
    }

    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        let sprite_meta = &mut sprite_meta;

        // Clear the vertex buffers
        sprite_meta.vertices.clear();
        sprite_meta.colored_vertices.clear();

        sprite_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_binding,
            }],
            label: Some("sprite_view_bind_group"),
            layout: &sprite_pipeline.view_layout,
        }));

        let draw_sprite_function = draw_functions.read().get_id::<DrawSprite>().unwrap();
        let key = SpritePipelineKey::from_msaa_samples(msaa.samples);
        let pipeline = pipelines.specialize(&mut pipeline_cache, &sprite_pipeline, key);
        let colored_pipeline = pipelines.specialize(
            &mut pipeline_cache,
            &sprite_pipeline,
            key | SpritePipelineKey::COLORED,
        );

        // Vertex buffer indices
        let mut index = 0;
        let mut colored_index = 0;

        // FIXME: VisibleEntities is ignored
        for mut transparent_phase in views.iter_mut() {
            let extracted_sprites = &mut extracted_sprites.sprites;
            let image_bind_groups = &mut *image_bind_groups;

            transparent_phase.items.reserve(extracted_sprites.len());

            // Sort sprites by z for correct transparency and then by handle to improve batching
            // TODO Sorting ignores color which may break batching
            extracted_sprites.sort_unstable_by(|a, b| {
                match a
                    .transform
                    .translation
                    .z
                    .partial_cmp(&b.transform.translation.z)
                {
                    Some(Ordering::Equal) | None => a.image_handle_id.cmp(&b.image_handle_id),
                    Some(other) => other,
                }
            });

            // Impossible starting values that will be replaced on the first iteration
            let mut current_batch = SpriteBatch {
                image_handle_id: HandleId::default::<Image>(),
                colored: false,
            };
            let mut current_batch_entity = Entity::new(u32::MAX);
            let mut current_image_size = Vec2::ZERO;
            // Add a phase item for each sprite, and detect when succesive items can be batched.
            // Spawn an entity with a `SpriteBatch` component for each possible batch.
            // Compatible items share the same entity.
            // Batches are merged later (in `batch_phase_system()`), so that they can be interrupted
            // by any other phase item (and they can interrupt other items from batching).
            for extracted_sprite in extracted_sprites.iter() {
                let new_batch = SpriteBatch {
                    image_handle_id: extracted_sprite.image_handle_id,
                    colored: extracted_sprite.color != Color::WHITE,
                };
                if new_batch != current_batch {
                    // Set-up a new possible batch
                    if let Some(gpu_image) =
                        gpu_images.get(&Handle::weak(new_batch.image_handle_id))
                    {
                        current_batch = new_batch;
                        current_image_size = gpu_image.size;
                        current_batch_entity = commands.spawn_bundle((current_batch,)).id();

                        image_bind_groups
                            .values
                            .entry(Handle::weak(current_batch.image_handle_id))
                            .or_insert_with(|| {
                                render_device.create_bind_group(&BindGroupDescriptor {
                                    entries: &[
                                        BindGroupEntry {
                                            binding: 0,
                                            resource: BindingResource::TextureView(
                                                &gpu_image.texture_view,
                                            ),
                                        },
                                        BindGroupEntry {
                                            binding: 1,
                                            resource: BindingResource::Sampler(&gpu_image.sampler),
                                        },
                                    ],
                                    label: Some("sprite_material_bind_group"),
                                    layout: &sprite_pipeline.material_layout,
                                })
                            });
                    } else {
                        // Skip this item if the texture is not ready
                        continue;
                    }
                }

                // Calculate vertex data for this item

                let mut uvs = QUAD_UVS;
                if extracted_sprite.flip_x {
                    uvs = [uvs[1], uvs[0], uvs[3], uvs[2]];
                }
                if extracted_sprite.flip_y {
                    uvs = [uvs[3], uvs[2], uvs[1], uvs[0]];
                }

                // By default, the size of the quad is the size of the texture
                let mut quad_size = current_image_size;

                // If a rect is specified, adjust UVs and the size of the quad
                if let Some(rect) = extracted_sprite.rect {
                    let rect_size = rect.size();
                    for uv in &mut uvs {
                        *uv = (rect.min + uv.mul_element_wise(rect_size))
                            .div_element_wise(current_image_size);
                    }
                    quad_size = rect_size;
                }

                // Override the size if a custom one is specified
                if let Some(custom_size) = extracted_sprite.custom_size {
                    quad_size = custom_size;
                }

                // Apply size and global transform
                let positions = QUAD_VERTEX_POSITIONS.map(|quad_pos| {
                    let pos = (quad_pos - extracted_sprite.anchor)
                        .mul_element_wise(quad_size)
                        .extend(0.);
                    (extracted_sprite.transform * pos).into()
                });

                // These items will be sorted by depth with other phase items
                let sort_key = FloatOrd(extracted_sprite.transform.translation.z);

                // Store the vertex data and add the item to the render phase
                if current_batch.colored {
                    let color = extracted_sprite.color.as_linear_rgba_f32();
                    // encode color as a single u32 to save space
                    let color = ((color[0] * 255.0) as u32)
                        | (((color[1] * 255.0) as u32) << 8)
                        | (((color[2] * 255.0) as u32) << 16)
                        | (((color[3] * 255.0) as u32) << 24);
                    for i in QUAD_INDICES.iter() {
                        sprite_meta.colored_vertices.push(ColoredSpriteVertex {
                            position: positions[*i],
                            uv: uvs[*i].into(),
                            color,
                        });
                    }
                    let item_start = colored_index;
                    colored_index += QUAD_INDICES.len() as u32;
                    let item_end = colored_index;

                    transparent_phase.add(Transparent2d {
                        draw_function: draw_sprite_function,
                        pipeline: colored_pipeline,
                        entity: current_batch_entity,
                        sort_key,
                        batch_range: Some(item_start..item_end),
                    });
                } else {
                    for i in QUAD_INDICES.iter() {
                        sprite_meta.vertices.push(SpriteVertex {
                            position: positions[*i],
                            uv: uvs[*i].into(),
                        });
                    }
                    let item_start = index;
                    index += QUAD_INDICES.len() as u32;
                    let item_end = index;

                    transparent_phase.add(Transparent2d {
                        draw_function: draw_sprite_function,
                        pipeline,
                        entity: current_batch_entity,
                        sort_key,
                        batch_range: Some(item_start..item_end),
                    });
                }
            }
        }
        sprite_meta
            .vertices
            .write_buffer(&render_device, &render_queue);
        sprite_meta
            .colored_vertices
            .write_buffer(&render_device, &render_queue);
    }
}

pub type DrawSprite = (
    SetItemPipeline,
    SetSpriteViewBindGroup<0>,
    SetSpriteTextureBindGroup<1>,
    DrawSpriteBatch,
);

pub struct SetSpriteViewBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetSpriteViewBindGroup<I> {
    type Param = (
        Res<'static, SpriteMeta>,
        Query<'static, 'static, &'static ViewUniformOffset>,
    );

    fn render<'w>(
        view: Entity,
        _item: Entity,
        (sprite_meta, view_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let view_uniform = view_query.get(view).unwrap();
        pass.set_bind_group(
            I,
            sprite_meta.into_inner().view_bind_group.as_ref().unwrap(),
            &[view_uniform.offset],
        );
        RenderCommandResult::Success
    }
}
pub struct SetSpriteTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetSpriteTextureBindGroup<I> {
    type Param = (
        Res<'static, ImageBindGroups>,
        Query<'static, 'static, &'static SpriteBatch>,
    );

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (image_bind_groups, query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_batch = query_batch.get(item).unwrap();
        let image_bind_groups = image_bind_groups.into_inner();

        pass.set_bind_group(
            I,
            image_bind_groups
                .values
                .get(&Handle::weak(sprite_batch.image_handle_id))
                .unwrap(),
            &[],
        );
        RenderCommandResult::Success
    }
}

pub struct DrawSpriteBatch;
impl<P: BatchedPhaseItem> RenderCommand<P> for DrawSpriteBatch {
    type Param = (
        Res<'static, SpriteMeta>,
        Query<'static, 'static, &'static SpriteBatch>,
    );

    fn render<'w>(
        _view: Entity,
        item: &P,
        (sprite_meta, query_batch): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let sprite_batch = query_batch.get(item.entity()).unwrap();
        let sprite_meta = sprite_meta.into_inner();
        if sprite_batch.colored {
            pass.set_vertex_buffer(0, sprite_meta.colored_vertices.buffer().unwrap().slice(..));
        } else {
            pass.set_vertex_buffer(0, sprite_meta.vertices.buffer().unwrap().slice(..));
        }
        pass.draw(item.batch_range().as_ref().unwrap().clone(), 0..1);
        RenderCommandResult::Success
    }
}
