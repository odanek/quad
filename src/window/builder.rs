use super::{Size, Window, WindowId};

#[derive(Default)]
pub struct WindowBuilder {
    title: String,
    size: Size,
}

impl WindowBuilder {
    pub fn title<T: Into<String>>(mut self, title: T) -> WindowBuilder {
        self.title = title.into();
        self
    }

    pub fn inner_size<T: Into<Size>>(mut self, size: T) -> WindowBuilder {
        self.size = size.into();
        self
    }

    pub(crate) fn build(
        &self,
        id: WindowId,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Window {
        let mut builder = winit::window::WindowBuilder::new().with_title(self.title.clone());

        match self.size {
            Size::Physical(size) => {
                builder =
                    builder.with_inner_size(winit::dpi::PhysicalSize::new(size.width, size.height));
            }
            Size::Logical(size) => {
                builder =
                    builder.with_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
            }
            Size::FullScreen(_) => {
                builder =
                    builder.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }
        }

        let winit_window = builder.build(event_loop).expect("Unable to create window");
        Window { id, winit_window }
    }
}
