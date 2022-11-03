mod bundle;
mod camera;
mod projection;

pub use bundle::*;
pub use camera::*;
pub use projection::*;

use crate::app::{App, MainStage};

pub fn camera_plugin(app: &mut App, render_app: &mut App) {
    app.add_system_to_stage(
        MainStage::PostTransformUpdate,
        camera_system::<OrthographicProjection>,
    );
    app.world
        .spawn()
        .insert_bundle(OrthographicCameraBundle::default()); // TODO Allow to customize the camera
    camera_type_plugin::<Camera2d>(app, render_app);
}
