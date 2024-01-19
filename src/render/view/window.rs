use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use crate::{
    app::{App, RenderStage},
    ecs::{Res, ResMut, Resource},
    render::{
        extract_param::Extract,
        render_resource::TextureView,
        renderer::{RenderDevice, RenderInstance},
        texture::TEXTURE_FORMAT,
    },
    windowing::{PresentMode, WindowHandleWrapper, WindowId, Windows},
};

pub fn window_render_plugin(_app: &mut App, render_app: &mut App) {
    render_app
        .init_resource::<ExtractedWindows>()
        .init_resource::<WindowSurfaces>()
        .add_system_to_stage(RenderStage::Extract, extract_windows)
        .add_system_to_stage(RenderStage::Prepare, prepare_windows);
}

pub struct ExtractedWindow {
    pub id: WindowId,
    pub handle: WindowHandleWrapper,
    pub physical_width: u32,
    pub physical_height: u32,
    pub present_mode: PresentMode,
    pub swap_chain_texture: Option<TextureView>,
    pub size_changed: bool,
}

#[derive(Default, Resource)]
pub struct ExtractedWindows {
    pub windows: HashMap<WindowId, ExtractedWindow>,
}

impl Deref for ExtractedWindows {
    type Target = HashMap<WindowId, ExtractedWindow>;

    fn deref(&self) -> &Self::Target {
        &self.windows
    }
}

impl DerefMut for ExtractedWindows {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.windows
    }
}

fn extract_windows(
    mut extracted_windows: ResMut<ExtractedWindows>,
    windows: Extract<Res<Windows>>,
) {
    for window in windows.iter() {
        let (new_width, new_height) = (
            window.physical_width().max(1),
            window.physical_height().max(1),
        );

        let extracted_window = extracted_windows
            .entry(window.id())
            .or_insert(ExtractedWindow {
                id: window.id(),
                handle: window.window_handle(),
                physical_width: new_width,
                physical_height: new_height,
                present_mode: window.present_mode(),
                swap_chain_texture: None,
                size_changed: false,
            });

        // NOTE: Drop the swap chain frame here
        extracted_window.swap_chain_texture = None;
        extracted_window.size_changed = new_width != extracted_window.physical_width
            || new_height != extracted_window.physical_height;

        if extracted_window.size_changed {
            log::debug!(
                "Window size changed from {}x{} to {}x{}",
                extracted_window.physical_width,
                extracted_window.physical_height,
                new_width,
                new_height
            );
            extracted_window.physical_width = new_width;
            extracted_window.physical_height = new_height;
        }
    }
}

#[derive(Default, Resource)]
pub struct WindowSurfaces {
    surfaces: HashMap<WindowId, wgpu::Surface<'static>>,
    /// List of windows that we have already called the initial `configure_surface` for
    configured_windows: HashSet<WindowId>,
}

// TODO Make sure this runs on the main thread (see create_surface call below)
pub fn prepare_windows(
    mut windows: ResMut<ExtractedWindows>,
    mut window_surfaces: ResMut<WindowSurfaces>,
    render_device: Res<RenderDevice>,
    render_instance: Res<RenderInstance>,
) {
    let window_surfaces = window_surfaces.deref_mut();
    for window in windows.windows.values_mut() {
        let surface = window_surfaces
            .surfaces
            .entry(window.id)
            .or_insert_with(|| {
                // NOTE: On some OSes this MUST be called from the main thread.
                render_instance
                    .create_surface(window.handle.clone())
                    .unwrap()
            });

        let swap_chain_descriptor = wgpu::SurfaceConfiguration {
            format: TEXTURE_FORMAT,
            width: window.physical_width,
            height: window.physical_height,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: match window.present_mode {
                PresentMode::Fifo => wgpu::PresentMode::Fifo,
                PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
                PresentMode::Immediate => wgpu::PresentMode::Immediate,
            },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Do the initial surface configuration if it hasn't been configured yet
        if window_surfaces.configured_windows.insert(window.id) || window.size_changed {
            render_device.configure_surface(surface, &swap_chain_descriptor);
        }

        let frame = match surface.get_current_texture() {
            Ok(swap_chain_frame) => swap_chain_frame,
            Err(wgpu::SurfaceError::Outdated) => {
                render_device.configure_surface(surface, &swap_chain_descriptor);
                surface
                    .get_current_texture()
                    .expect("Error reconfiguring surface")
            }
            err => err.expect("Failed to acquire next swap chain texture!"),
        };

        window.swap_chain_texture = Some(TextureView::from(frame));
    }
}
