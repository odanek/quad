//! This module contains the bundles used in Bevy's UI

use crate::{
    ecs::{Bundle, Component},
    render::{
        cameras::{Camera, DepthCalculation, OrthographicProjection, WindowOrigin},
        view::{ComputedVisibility, Visibility, VisibleEntities},
    },
    text::Text,
    transform::{GlobalTransform, Transform},
};

use super::{
    BackgroundColor, CalculatedSize, FocusPolicy, Interaction, Node, Style, UiImage,
    widget::{Button, ImageMode},
};

/// The basic UI node
#[derive(Bundle, Clone, Debug, Default)]
pub struct NodeBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// Describes the color of the node
    pub color: BackgroundColor,
    /// Describes the image of the node
    pub image: UiImage,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The transform of the node
    pub transform: Transform,
    /// The global transform of the node
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

/// A UI node that is an image
#[derive(Bundle, Clone, Debug, Default)]
pub struct ImageBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// Configures how the image should scale
    pub image_mode: ImageMode,
    /// The calculated size based on the given image
    pub calculated_size: CalculatedSize,
    /// The color of the node
    pub background_color: BackgroundColor,
    /// The image of the node
    pub image: UiImage,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The transform of the node
    pub transform: Transform,
    /// The global transform of the node
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

/// A UI node that is text
#[derive(Bundle, Clone, Debug)]
pub struct UiTextBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// Contains the text of the node
    pub text: Text,
    /// The calculated size based on the given image
    pub calculated_size: CalculatedSize,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The transform of the node
    pub transform: Transform,
    /// The global transform of the node
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

impl Default for UiTextBundle {
    fn default() -> Self {
        Self {
            focus_policy: FocusPolicy::Pass,
            text: Default::default(),
            node: Default::default(),
            calculated_size: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}

/// A UI node that is a button
#[derive(Bundle, Clone, Debug, Default)]
pub struct ButtonBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Marker component that signals this node is a button
    pub button: Button,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// Describes whether and how the button has been interacted with by the input
    pub interaction: Interaction,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// When combined with `UiImage`, tints the provided image.
    pub background_color: BackgroundColor,
    /// The image of the node
    pub image: UiImage,
    /// The transform of the node
    pub transform: Transform,
    /// The global transform of the node
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

#[derive(Component, Default)]
pub struct CameraUi;

/// The camera that is needed to see UI elements
#[derive(Bundle, Debug)]
pub struct UiCameraBundle<M: Component> {
    /// The camera component
    pub camera: Camera,
    /// The orthographic projection settings
    pub orthographic_projection: OrthographicProjection,
    /// The transform of the camera
    pub transform: Transform,
    /// The global transform of the camera
    pub global_transform: GlobalTransform,
    /// Contains visible entities
    // FIXME there is no frustrum culling for UI
    pub visible_entities: VisibleEntities,
    pub marker: M,
}

impl Default for UiCameraBundle<CameraUi> {
    fn default() -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system
        let far = 1000.0;
        UiCameraBundle {
            camera: Camera {
                ..Default::default()
            },
            orthographic_projection: OrthographicProjection {
                far,
                window_origin: WindowOrigin::BottomLeft,
                depth_calculation: DepthCalculation::ZDifference,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, far - 0.1),
            global_transform: Default::default(),
            visible_entities: Default::default(),
            marker: CameraUi,
        }
    }
}
