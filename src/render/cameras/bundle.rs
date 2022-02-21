use crate::{
    render::primitives::Frustum,
    transform::{GlobalTransform, Transform},
    ty::Vec3,
};
use cgm::Zero;

use super::{Camera, CameraProjection, PerspectiveProjection, OrthographicProjection, DepthCalculation, ScalingMode, CAMERA_2D, CAMERA_3D};

/// Component bundle for camera entities with perspective projection
///
/// Use this for 3D rendering.
#[derive(Bundle)]
pub struct PerspectiveCameraBundle {
    pub camera: Camera,
    pub perspective_projection: PerspectiveProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl PerspectiveCameraBundle {
    pub fn new_3d() -> Self {
        Default::default()
    }

    pub fn with_name(name: &str) -> Self {
        let perspective_projection = PerspectiveProjection::default();
        let view_projection = perspective_projection.get_projection_matrix();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &Vec3::ZERO,
            &Vec3::Z,
            perspective_projection.far(),
        );
        PerspectiveCameraBundle {
            camera: Camera {
                name: Some(name.to_string()),
                near: perspective_projection.near,
                far: perspective_projection.far,
                ..Default::default()
            },
            perspective_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

impl Default for PerspectiveCameraBundle {
    fn default() -> Self {
        PerspectiveCameraBundle::with_name(CAMERA_3D)
    }
}

/// Component bundle for camera entities with orthographic projection
///
/// Use this for 2D games, isometric games, CAD-like 3D views.
#[derive(Bundle)]
pub struct OrthographicCameraBundle {
    pub camera: Camera,
    pub orthographic_projection: OrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl OrthographicCameraBundle {
    /// Create an orthographic projection camera to render 2D content.
    ///
    /// The projection creates a camera space where X points to the right of the screen,
    /// Y points to the top of the screen, and Z points out of the screen (backward),
    /// forming a right-handed coordinate system. The center of the screen is at `X=0` and
    /// `Y=0`.
    ///
    /// The default scaling mode is [`ScalingMode::WindowSize`], resulting in a resolution
    /// where 1 unit in X and Y in camera space corresponds to 1 logical pixel on the screen.
    /// That is, for a screen of 1920 pixels in width, the X coordinates visible on screen go
    /// from `X=-960` to `X=+960` in world space, left to right. This can be changed by changing
    /// the [`OrthographicProjection::scaling_mode`] field.
    ///
    /// The camera is placed at `Z=+1000-0.1`, looking toward the world origin `(0,0,0)`.
    /// Its orthographic projection extends from `0.0` to `-1000.0` in camera view space,
    /// corresponding to `Z=+999.9` (closest to camera) to `Z=-0.1` (furthest away from
    /// camera) in world space.
    pub fn new_2d() -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system
        let far = 1000.0;
        let orthographic_projection = OrthographicProjection {
            far,
            depth_calculation: DepthCalculation::ZDifference,
            ..Default::default()
        };
        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
        let view_projection =
            orthographic_projection.get_projection_matrix() * transform.compute_matrix().inverse();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            orthographic_projection.far(),
        );
        OrthographicCameraBundle {
            camera: Camera {
                name: Some(CAMERA_2D.to_string()),
                near: orthographic_projection.near,
                far: orthographic_projection.far,
                ..Default::default()
            },
            orthographic_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform,
            global_transform: Default::default(),
        }
    }

    pub fn new_3d() -> Self {
        let orthographic_projection = OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical,
            depth_calculation: DepthCalculation::Distance,
            ..Default::default()
        };
        let view_projection = orthographic_projection.get_projection_matrix();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &Vec3::ZERO,
            &Vec3::Z,
            orthographic_projection.far(),
        );
        OrthographicCameraBundle {
            camera: Camera {
                name: Some(CAMERA_3D.to_string()),
                near: orthographic_projection.near,
                far: orthographic_projection.far,
                ..Default::default()
            },
            orthographic_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }

    pub fn with_name(name: &str) -> Self {
        let orthographic_projection = OrthographicProjection::default();
        let view_projection = orthographic_projection.get_projection_matrix();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &Vec3::ZERO,
            &Vec3::Z,
            orthographic_projection.far(),
        );
        OrthographicCameraBundle {
            camera: Camera {
                name: Some(name.to_string()),
                near: orthographic_projection.near,
                far: orthographic_projection.far,
                ..Default::default()
            },
            orthographic_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}
