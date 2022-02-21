mod active_cameras;
mod bundle;
mod camera;
mod projection;

use std::collections::HashMap;

pub use active_cameras::*;
pub use bundle::*;
pub use camera::*;
pub use projection::*;

use crate::{
    app::{App, Stage},
    ecs::{Commands, Component, Entity, Query, Res, Resource},
    transform::GlobalTransform,
    windowing::{WindowId, Windows},
};

pub const CAMERA_2D: &'static str = "camera_2d";
pub const CAMERA_3D: &'static str = "camera_3d";

pub fn camera_plugin(app: &mut App, render_app: &mut App) {
    let mut active_cameras = ActiveCameras::default();
    active_cameras.add(CAMERA_2D);
    active_cameras.add(CAMERA_3D);
    app.insert_resource(active_cameras)
        .add_system_to_stage(Stage::PostUpdate, &active_cameras_system)
        .add_system_to_stage(Stage::PostUpdate, &camera_system::<OrthographicProjection>)
        .add_system_to_stage(Stage::PostUpdate, &camera_system::<PerspectiveProjection>);
    render_app
        .init_resource::<ExtractedCameraNames>()
        .add_system_to_stage(Stage::RenderExtract, &extract_cameras);
}

#[derive(Resource, Default)]
pub struct ExtractedCameraNames {
    pub entities: HashMap<String, Entity>,
}

#[derive(Component, Debug)]
pub struct ExtractedCamera {
    pub window_id: WindowId,
    pub name: Option<String>,
}

fn extract_cameras(
    mut commands: Commands,
    active_cameras: Res<ActiveCameras>,
    windows: Res<Windows>,
    query: Query<(Entity, &Camera, &GlobalTransform, &VisibleEntities)>,
) {
    let mut entities = HashMap::default();
    for camera in active_cameras.iter() {
        let name = &camera.name;
        if let Some((entity, camera, transform, visible_entities)) =
            camera.entity.and_then(|e| query.get(e).ok())
        {
            if let Some(window) = windows.get(camera.window) {
                entities.insert(name.clone(), entity);
                commands.get_or_spawn(entity).insert_bundle((
                    ExtractedCamera {
                        window_id: camera.window,
                        name: camera.name.clone(),
                    },
                    ExtractedView {
                        projection: camera.projection_matrix,
                        transform: *transform,
                        width: window.physical_width().max(1),
                        height: window.physical_height().max(1),
                        near: camera.near,
                        far: camera.far,
                    },
                    visible_entities.clone(),
                ));
            }
        }
    }

    commands.insert_resource(ExtractedCameraNames { entities })
}
