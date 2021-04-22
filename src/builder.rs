use crate::{Quad, WindowBuilder};

pub struct QuadBuilder {
    main_window: WindowBuilder,
}

impl Default for QuadBuilder {
    fn default() -> Self {
        QuadBuilder {
            main_window: WindowBuilder::default(),
        }
    }
}

impl QuadBuilder {
    pub fn main_window(mut self, window: WindowBuilder) -> QuadBuilder {
        self.main_window = window;
        self
    }

    pub fn build(self) -> Quad {
        Quad {
            main_window: self.main_window.build(),
        }
    }
}
