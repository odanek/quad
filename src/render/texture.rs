mod image;
mod image_texture_loader;
mod texture_cache;

pub(crate) mod image_texture_conversion;

use crate::{
    app::{App, Stage},
    asset::Assets,
};

pub use self::image::*;
pub use image_texture_loader::*;
pub use texture_cache::*;

use super::render_asset::render_asset_plugin;

// TODO: Use Plugin trait like Bevy? How to deal with plugins requiring the RenderApp?
pub fn image_plugin(app: &mut App, render_app: &mut App) {
    app.init_asset_loader::<ImageTextureLoader>();
    app.add_asset::<Image>();
    app.world
        .resource_mut::<Assets<Image>>()
        .set_untracked(DEFAULT_IMAGE_HANDLE, Image::default());

    render_asset_plugin::<Image>(render_app);
    render_app
        .init_resource::<TextureCache>()
        .add_system_to_stage(Stage::RenderCleanup, &update_texture_cache_system);
}

// TODO Rename to QuadDefault?
pub trait BevyDefault {
    fn bevy_default() -> Self;
}

impl BevyDefault for wgpu::TextureFormat {
    fn bevy_default() -> Self {
        if cfg!(target_os = "android") || cfg!(target_arch = "wasm32") {
            // Bgra8UnormSrgb texture missing on some Android devices
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Bgra8UnormSrgb
        }
    }
}
