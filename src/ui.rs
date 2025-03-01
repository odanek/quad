mod flex;
mod focus;
mod geometry;
mod render;
mod ui_node;

pub mod entity;
pub mod update;
pub mod widget;

pub use flex::*;
pub use focus::*;
pub use geometry::*;
pub use render::*;
pub use ui_node::*;

use update::update_clipping_system;

use crate::{
    app::{App, MainStage},
    ecs::Resource,
    render::cameras::camera_type_plugin,
};

use self::{
    entity::{CameraUi, UiCameraBundle},
    update::ui_z_system,
};

pub mod prelude {
    pub use crate::ui::{
        AlignItems, FlexDirection, JustifyContent, PositionType, Size, Style, UiRect, Val,
        ValArithmeticError,
        entity::{NodeBundle, UiTextBundle},
    };
}

#[derive(Debug, Resource)]
pub struct UiScale {
    /// The scale to be applied.
    pub scale: f64,
}

impl Default for UiScale {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

pub fn ui_plugin(app: &mut App, render_app: &mut App) {
    app.world.spawn().insert_bundle(UiCameraBundle::default()); // TODO Allow to customize the camera
    camera_type_plugin::<CameraUi>(app, render_app);

    app.init_resource::<FlexSurface>()
        .init_resource::<UiScale>()
        .add_system_to_stage(
            MainStage::PreUpdate, // After input systems
            ui_focus_system,
        )
        .add_system_to_stage(
            MainStage::PreTransformUpdate, // Before flex_node_system after modifies_window
            widget::text_system,
        )
        .add_system_to_stage(
            MainStage::PreTransformUpdate, // Before flex_node_system
            widget::update_image_calculated_size_system,
        )
        .add_system_to_stage(
            MainStage::PreTransformUpdate, // Before transform_propagate_system, after modifies_windows
            flex_node_system,
        )
        .add_system_to_stage(
            MainStage::PreTransformUpdate, // Before transform_propagate_system, after flex_node_system
            ui_z_system,
        )
        .add_system_to_stage(
            MainStage::PostTransformUpdate, // After transform_propagate_system
            update_clipping_system,
        );

    build_ui_render(app, render_app);
}
