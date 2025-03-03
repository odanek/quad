pub mod visibility;
pub mod window;

use cgm::SquareMatrix;
use crevice::std140::AsStd140;
pub use visibility::*;
use wgpu::{
    Color, Extent3d, Operations, RenderPassColorAttachment, TextureDescriptor, TextureDimension,
    TextureUsages,
};

use crate::{
    app::{App, RenderStage},
    ecs::{Commands, Component, Entity, Query, Res, ResMut, Resource},
    transform::GlobalTransform,
    ty::{Mat4, Vec3},
};

use self::window::ExtractedWindows;

use super::{
    cameras::ExtractedCamera,
    extract_param::Extract,
    render_asset::RenderAssets,
    render_resource::{DynamicUniformVec, Texture, TextureView},
    renderer::{RenderDevice, RenderQueue},
    texture::{Image, TEXTURE_FORMAT, TextureCache},
};

pub fn view_plugin(app: &mut App, render_app: &mut App) {
    app.init_resource::<Msaa>();
    visibility_plugin(app);
    render_app
        .init_resource::<ViewUniforms>()
        .add_system_to_stage(RenderStage::Extract, extract_msaa)
        .add_system_to_stage(RenderStage::Prepare, prepare_view_uniforms)
        .add_system_to_stage(RenderStage::Prepare, prepare_view_targets); // Must run after prepare_windows
}

#[derive(Clone)]
/// Configuration resource for [Multi-Sample Anti-Aliasing](https://en.wikipedia.org/wiki/Multisample_anti-aliasing).
#[derive(Resource)]
pub struct Msaa {
    /// The number of samples to run for Multi-Sample Anti-Aliasing. Higher numbers result in
    /// smoother edges.
    /// Defaults to 4.
    ///
    /// Note that WGPU currently only supports 1 or 4 samples.
    /// Ultimately we plan on supporting whatever is natively supported on a given device.
    /// Check out this issue for more info: <https://github.com/gfx-rs/wgpu/issues/1832>
    pub samples: u32,
}

impl Default for Msaa {
    fn default() -> Self {
        Self { samples: 4 }
    }
}

pub fn extract_msaa(mut commands: Commands, msaa: Extract<Res<Msaa>>) {
    // NOTE: windows.is_changed() handles cases where a window was resized
    commands.insert_resource(msaa.clone());
}

#[derive(Component)]
pub struct ExtractedView {
    pub projection: Mat4,
    pub transform: GlobalTransform,
    pub width: u32,
    pub height: u32,
    pub near: f32,
    pub far: f32,
}

#[derive(Clone, AsStd140)]
pub struct ViewUniform {
    view_proj: Mat4,
    view: Mat4,
    inverse_view: Mat4,
    projection: Mat4,
    world_position: Vec3,
    near: f32,
    far: f32,
    width: f32,
    height: f32,
}

#[derive(Default, Resource)]
pub struct ViewUniforms {
    pub uniforms: DynamicUniformVec<ViewUniform>,
}

#[derive(Component)]
pub struct ViewUniformOffset {
    pub offset: u32,
}

#[derive(Component)]
pub struct ViewTarget {
    pub view: TextureView,
    pub sampled_target: Option<TextureView>,
}

impl ViewTarget {
    pub fn get_color_attachment(&self, ops: Operations<Color>) -> RenderPassColorAttachment {
        RenderPassColorAttachment {
            view: if let Some(sampled_target) = &self.sampled_target {
                sampled_target
            } else {
                &self.view
            },
            resolve_target: if self.sampled_target.is_some() {
                Some(&self.view)
            } else {
                None
            },
            ops,
        }
    }
}

#[derive(Component)]
pub struct ViewDepthTexture {
    pub texture: Texture,
    pub view: TextureView,
}

fn prepare_view_uniforms(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut view_uniforms: ResMut<ViewUniforms>,
    views: Query<(Entity, &ExtractedView)>,
) {
    view_uniforms.uniforms.clear();
    for (entity, camera) in views.iter() {
        let projection = camera.projection;
        let view = camera.transform.compute_matrix();
        let inverse_view = view.inverse().unwrap();
        let view_uniforms = ViewUniformOffset {
            offset: view_uniforms.uniforms.push(ViewUniform {
                view_proj: projection * inverse_view,
                view,
                inverse_view,
                projection,
                world_position: camera.transform.translation,
                near: camera.near,
                far: camera.far,
                width: camera.width as f32,
                height: camera.height as f32,
            }),
        };

        commands.entity(entity).insert(view_uniforms);
    }

    view_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}

#[allow(clippy::too_many_arguments)]
fn prepare_view_targets(
    mut commands: Commands,
    windows: Res<ExtractedWindows>,
    images: Res<RenderAssets<Image>>,
    msaa: Res<Msaa>,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    cameras: Query<(Entity, &ExtractedCamera)>,
) {
    for (entity, camera) in cameras.iter() {
        if let Some(size) = camera.physical_size {
            if let Some(texture_view) = camera.target.get_texture_view(&windows, &images) {
                let sampled_target = if msaa.samples > 1 {
                    let sampled_texture = texture_cache.get(
                        &render_device,
                        TextureDescriptor {
                            label: Some("sampled_color_attachment_texture"),
                            size: Extent3d {
                                width: size.x,
                                height: size.y,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: msaa.samples,
                            dimension: TextureDimension::D2,
                            format: TEXTURE_FORMAT,
                            usage: TextureUsages::RENDER_ATTACHMENT,
                            view_formats: &[],
                        },
                    );
                    Some(sampled_texture.default_view.clone())
                } else {
                    None
                };
                commands.entity(entity).insert(ViewTarget {
                    view: texture_view.clone(),
                    sampled_target,
                });
            }
        }
    }
}
