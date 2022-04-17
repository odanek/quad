use crate::{
    ecs::{Bundle, Component},
    render::{primitives::Frustum, view::VisibleEntities},
    transform::{GlobalTransform, Transform},
};

use super::{Camera, CameraProjection, DepthCalculation, OrthographicProjection};
use cgm::SquareMatrix;

#[derive(Component, Default)]
pub struct Camera2d;

/// Component bundle for camera entities with orthographic projection
///
/// Use this for 2D games, isometric games, CAD-like 3D views.
#[derive(Bundle)]
pub struct OrthographicCameraBundle<M: Component> {
    pub camera: Camera,
    pub orthographic_projection: OrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub marker: M,
}

impl Default for OrthographicCameraBundle<Camera2d> {
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
    fn default() -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system
        let far = 1000.0;
        let orthographic_projection = OrthographicProjection {
            far,
            depth_calculation: DepthCalculation::ZDifference,
            ..Default::default()
        };
        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
        // TODO No need to call inverse here
        let view_projection = orthographic_projection.get_projection_matrix()
            * transform.compute_matrix().inverse().unwrap();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            orthographic_projection.far(),
        );
        OrthographicCameraBundle {
            camera: Camera {
                near: orthographic_projection.near,
                far: orthographic_projection.far,
                ..Default::default()
            },
            orthographic_projection,
            visible_entities: VisibleEntities::default(),
            frustum,
            transform,
            global_transform: Default::default(),
            marker: Camera2d,
        }
    }
}
