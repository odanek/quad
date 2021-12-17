use crate::{
    app::{winit_runner, App, AppContext, Scene},
    windowing::{WindowBuilder, WindowId},
};

pub struct Quad {
    app: App,
    main_window: WindowBuilder,
}

impl Default for Quad {
    fn default() -> Self {
        let mut quad = Self {
            app: App::default(),
            main_window: WindowBuilder::default(),
        };
        quad.add_default_pools();
        quad.add_default_plugins();
        quad
    }
}

impl Quad {
    fn add_default_pools(&mut self) {
        self.app.create_default_pools();
    }

    fn add_default_plugins(&mut self) {
        self.app.add_timing_plugin();
        self.app.add_windowing_plugin();
        self.app.add_input_plugin();
        self.app.add_asset_plugin();
    }

    pub fn main_window(&mut self, window: WindowBuilder) -> &mut Self {
        self.main_window = window;
        self
    }

    pub fn run(self, scene: Box<dyn Scene>) {
        let mut app = self.app;
        let event_loop = winit::event_loop::EventLoop::new();
        let main_window = self.main_window.build(WindowId::new(0), &event_loop);
        app.add_window(main_window);
        let context = AppContext::new(app, scene);
        winit_runner(context, event_loop);
    }
}
