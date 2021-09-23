use crate::window::WindowBuilder;

use super::App;

pub struct AppBuilder {
    main_window: WindowBuilder,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            main_window: WindowBuilder::default(),
        }
    }
}

impl AppBuilder {
    pub fn main_window(mut self, window: WindowBuilder) -> AppBuilder {
        self.main_window = window;
        self
    }

    pub fn build(self) -> App {
        App {
            main_window: self.main_window.build(),
        }
    }
}
