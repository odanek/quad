use std::{collections::HashSet, marker::PhantomData};

use cgm::{ElementWise, SquareMatrix};
use wgpu::Extent3d;

use crate::{
    app::{App, MainStage, RenderStage},
    asset::{AssetEvent, Assets, Handle},
    ecs::{
        Added, Commands, Component, DetectChanges, Entity, EventReader, ParamSet, Query, Res,
        ResMut, Resource, With,
    },
    render::{
        extract_param::Extract,
        render_asset::RenderAssets,
        render_resource::TextureView,
        texture::Image,
        view::{ExtractedView, VisibleEntities, window::ExtractedWindows},
    },
    transform::GlobalTransform,
    ty::{Mat4, Vec2, Vec2u, Vec3},
    windowing::{WindowCreated, WindowId, WindowResized, Windows},
};

use super::CameraProjection;

#[derive(Component, Default, Debug)]
pub struct Camera {
    pub projection_matrix: Mat4,
    pub target: RenderTarget,
    pub depth_calculation: DepthCalculation,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenderTarget {
    /// Window to which the camera's view is rendered.
    Window(WindowId),
    /// Image to which the camera's view is rendered.
    Image(Handle<Image>),
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Window(Default::default())
    }
}

impl RenderTarget {
    pub fn get_texture_view<'a>(
        &self,
        windows: &'a ExtractedWindows,
        images: &'a RenderAssets<Image>,
    ) -> Option<&'a TextureView> {
        match self {
            RenderTarget::Window(window_id) => windows
                .get(window_id)
                .and_then(|window| window.swap_chain_texture.as_ref()),
            RenderTarget::Image(image_handle) => {
                images.get(image_handle).map(|image| &image.texture_view)
            }
        }
    }
    pub fn get_physical_size(&self, windows: &Windows, images: &Assets<Image>) -> Option<Vec2u> {
        match self {
            RenderTarget::Window(window_id) => windows
                .get(*window_id)
                .map(|window| Vec2u::new(window.physical_width(), window.physical_height())),
            RenderTarget::Image(image_handle) => images.get(image_handle).map(|image| {
                let Extent3d { width, height, .. } = image.texture_descriptor.size;
                Vec2u::new(width, height)
            }),
        }
    }
    pub fn get_logical_size(&self, windows: &Windows, images: &Assets<Image>) -> Option<Vec2> {
        match self {
            RenderTarget::Window(window_id) => windows
                .get(*window_id)
                .map(|window| Vec2::new(window.width(), window.height())),
            RenderTarget::Image(image_handle) => images.get(image_handle).map(|image| {
                let Extent3d { width, height, .. } = image.texture_descriptor.size;
                Vec2::new(width as f32, height as f32)
            }),
        }
    }
    // Check if this render target is contained in the given changed windows or images.
    fn is_changed(
        &self,
        changed_window_ids: &[WindowId],
        changed_image_handles: &HashSet<&Handle<Image>>,
    ) -> bool {
        match self {
            RenderTarget::Window(window_id) => changed_window_ids.contains(window_id),
            RenderTarget::Image(image_handle) => changed_image_handles.contains(&image_handle),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum DepthCalculation {
    /// Pythagorean distance; works everywhere, more expensive to compute.
    #[default]
    Distance,
    /// Optimization for 2D; assuming the camera points towards -Z.
    ZDifference,
}

impl Camera {
    /// Given a position in world space, use the camera to compute the screen space coordinates.
    pub fn world_to_screen(
        &self,
        windows: &Windows,
        images: &Assets<Image>,
        camera_transform: &GlobalTransform,
        world_position: Vec3,
    ) -> Option<Vec2> {
        let window_size = self.target.get_logical_size(windows, images)?;
        // Build a transform to convert from world to NDC using camera data
        let world_to_ndc: Mat4 =
            self.projection_matrix * camera_transform.compute_matrix().inverse().unwrap();
        let ndc_space_coords: Vec3 =
            Vec3::from_homogeneous(world_to_ndc * world_position.to_homogeneous());
        // NDC z-values outside of 0 < z < 1 are outside the camera frustum and are thus not in screen space
        if ndc_space_coords.z < 0.0 || ndc_space_coords.z > 1.0 {
            return None;
        }
        // Once in NDC space, we can discard the z element and rescale x/y to fit the screen
        let screen_space_coords = ((ndc_space_coords.truncate() + Vec2::new(1.0, 1.0)) / 2.0)
            .mul_element_wise(window_size);
        Some(screen_space_coords)

        // TODO
        // if !screen_space_coords.is_nan() {
        //     Some(screen_space_coords)
        // } else {
        //     None
        // }
    }
}

#[allow(clippy::type_complexity)]
pub fn camera_system<T: CameraProjection + Component + 'static>(
    mut window_resized_events: EventReader<WindowResized>,
    mut window_created_events: EventReader<WindowCreated>,
    mut image_asset_events: EventReader<AssetEvent<Image>>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    mut queries: ParamSet<(
        Query<(Entity, &'static mut Camera, &'static mut T)>,
        Query<Entity, Added<Camera>>,
    )>,
) {
    let mut changed_window_ids = Vec::new();
    // handle resize events. latest events are handled first because we only want to resize each
    // window once
    for event in window_resized_events.iter().rev() {
        if changed_window_ids.contains(&event.id) {
            continue;
        }

        changed_window_ids.push(event.id);
    }

    // handle resize events. latest events are handled first because we only want to resize each
    // window once
    for event in window_created_events.iter().rev() {
        if changed_window_ids.contains(&event.id) {
            continue;
        }

        changed_window_ids.push(event.id);
    }

    let changed_image_handles: HashSet<&Handle<Image>> = image_asset_events
        .iter()
        .filter_map(|event| {
            if let AssetEvent::Modified { handle } = event {
                Some(handle)
            } else {
                None
            }
        })
        .collect();

    let mut added_cameras = vec![];
    for entity in &mut queries.p1().iter() {
        added_cameras.push(entity);
    }
    for (entity, mut camera, mut camera_projection) in queries.p0().iter_mut() {
        if camera
            .target
            .is_changed(&changed_window_ids, &changed_image_handles)
            || added_cameras.contains(&entity)
            || camera_projection.is_changed()
        {
            if let Some(size) = camera.target.get_logical_size(&windows, &images) {
                camera_projection.update(size.x, size.y);
                camera.projection_matrix = camera_projection.get_projection_matrix();
                camera.depth_calculation = camera_projection.depth_calculation();
            }
        }
    }
}

pub fn camera_type_plugin<T: Component + Default>(app: &mut App, render_app: &mut App) {
    app.init_resource::<ActiveCamera<T>>()
        // TODO Initialize the camera
        //.add_startup_system_to_stage(StartupStage::PostStartup, set_active_camera::<T>)
        .add_system_to_stage(MainStage::PostTransformUpdate, set_active_camera::<T>);
    render_app.add_system_to_stage(RenderStage::Extract, extract_cameras::<T>);
}

/// The canonical source of the "active camera" of the given camera type `T`.
#[derive(Debug, Resource)]
pub struct ActiveCamera<T: Component> {
    camera: Option<Entity>,
    marker: PhantomData<T>,
}

impl<T: Component> Default for ActiveCamera<T> {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            marker: Default::default(),
        }
    }
}

impl<T: Component> Clone for ActiveCamera<T> {
    fn clone(&self) -> Self {
        Self {
            camera: self.camera,
            marker: self.marker,
        }
    }
}

impl<T: Component> ActiveCamera<T> {
    /// Sets the active camera to the given `camera` entity.
    pub fn set(&mut self, camera: Entity) {
        self.camera = Some(camera);
    }

    /// Returns the active camera, if it exists.
    pub fn get(&self) -> Option<Entity> {
        self.camera
    }
}

pub fn set_active_camera<T: Component>(
    mut active_camera: ResMut<ActiveCamera<T>>,
    cameras: Query<Entity, With<T>>,
) {
    if active_camera.get().is_some() {
        return;
    }

    if let Some(camera) = cameras.iter().next() {
        active_camera.camera = Some(camera);
    }
}

#[derive(Component, Debug)]
pub struct ExtractedCamera {
    pub target: RenderTarget,
    pub physical_size: Option<Vec2u>,
}

#[allow(clippy::type_complexity)]
pub fn extract_cameras<M: Component + Default>(
    mut commands: Commands,
    windows: Extract<Res<Windows>>,
    images: Extract<Res<Assets<Image>>>,
    active_camera: Extract<Res<ActiveCamera<M>>>,
    query: Extract<Query<(&Camera, &GlobalTransform, &VisibleEntities), With<M>>>,
) {
    if let Some(entity) = active_camera.get() {
        if let Ok((camera, transform, visible_entities)) = query.get(entity) {
            if let Some(size) = camera.target.get_physical_size(&windows, &images) {
                commands.get_or_spawn(entity).insert_bundle((
                    ExtractedCamera {
                        target: camera.target.clone(),
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
                    M::default(),
                ));
            }
        }
    }

    commands.insert_resource(active_camera.clone())
}
