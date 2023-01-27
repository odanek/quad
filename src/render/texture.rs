mod image;
mod image_texture_loader;
mod texture_cache;

pub(crate) mod image_texture_conversion;

use crate::{
    app::{App, RenderStage},
    asset::{Assets, HandleId},
};

pub use self::image::*;
pub use image_texture_loader::*;
pub use texture_cache::*;

use super::render_asset::render_asset_plugin;

// TODO: Use Plugin trait like Bevy? How to deal with plugins requiring the RenderApp?
pub fn image_plugin(app: &mut App, render_app: &mut App) {
    app.init_asset_loader::<ImageTextureLoader>();
    app.add_asset::<Image>();
    app.world.resource_mut::<Assets<Image>>().set_untracked(
        HandleId::with_id::<Image>(DEFAULT_IMAGE_HANDLE),
        Image::default(),
    );

    render_asset_plugin::<Image>(app, render_app);
    render_app
        .init_resource::<TextureCache>()
        .add_system_to_stage(RenderStage::Cleanup, update_texture_cache_system);
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
