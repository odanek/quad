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
    asset::Assets,
    ecs::{Commands, Component, Entity, Query, Res, Resource},
    transform::GlobalTransform,
    ty::Vec2u,
    windowing::Windows,
};

use super::{
    texture::Image,
    view::{ExtractedView, VisibleEntities},
};

pub const CAMERA_2D: &str = "camera_2d";
pub const CAMERA_3D: &str = "camera_3d";

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
    pub target: RenderTarget,
    pub name: Option<String>,
    pub physical_size: Option<Vec2u>,
}

fn extract_cameras(
    mut commands: Commands,
    active_cameras: Res<ActiveCameras>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &Camera, &GlobalTransform, &VisibleEntities)>,
) {
    let mut entities = HashMap::default();
    for camera in active_cameras.iter() {
        let name = &camera.name;
        if let Some((entity, camera, transform, visible_entities)) =
            camera.entity.and_then(|e| query.get(e).ok())
        {
            if let Some(size) = camera.target.get_physical_size(&windows, &images) {
                entities.insert(name.clone(), entity);
                commands.get_or_spawn(entity).insert_bundle((
                    ExtractedCamera {
                        target: camera.target.clone(),
                        name: camera.name.clone(),
                        physical_size: camera.target.get_physical_size(&windows, &images),
                    },
                    ExtractedView {
                        projection: camera.projection_matrix,
                        transform: *transform,
                        width: size.x.max(1),
                        height: size.y.max(1),
                        near: camera.near,
                        far: camera.far,
                    },
                    visible_entities.clone(),
                ));
            }
        }
    }

    commands.insert_resource(ExtractedCameraNames { entities });
}
