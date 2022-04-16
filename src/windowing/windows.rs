use std::collections::HashMap;

use crate::ecs::Resource;

use super::{Window, WindowId};

type WinitWindowId = winit::window::WindowId;

// TODO Window commands

#[derive(Default, Resource)]
pub struct Windows {
    window_id_to_winit: HashMap<WindowId, WinitWindowId>,
    winit_to_window_id: HashMap<WinitWindowId, WindowId>,
    windows: HashMap<WindowId, Window>,
}

impl Windows {
    pub(crate) fn get_id(&self, id: WinitWindowId) -> Option<WindowId> {
        self.winit_to_window_id.get(&id).copied()
    }

    pub fn get(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    pub fn get_primary(&self) -> Option<&Window> {
        self.get(WindowId::primary())
    }

    pub fn primary(&self) -> &Window {
        self.get(WindowId::primary())
            .expect("Main window not found")
    }

    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    pub fn add(&mut self, window: Window) {
        let id = window.id();
        let winit_id = window.winit_id();

        self.windows.insert(id, window);
        self.window_id_to_winit.insert(id, winit_id);
        self.winit_to_window_id.insert(winit_id, id);
    }

    pub fn scale_factor(&self, id: WindowId) -> f64 {
        if let Some(window) = self.get(id) {
            window.scale_factor()
        } else {
            1.0
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Window> {
        self.windows.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        self.windows.values_mut()
    }
}
