mod bundle;
mod camera;
mod projection;

pub use bundle::*;
pub use camera::*;
pub use projection::*;

use crate::app::{App, Stage};

pub fn camera_plugin(app: &mut App, render_app: &mut App) {
    app.add_system_to_stage(Stage::PostUpdate, &camera_system::<OrthographicProjection>);
    camera_type_plugin::<Camera2d>(app, render_app);
}
