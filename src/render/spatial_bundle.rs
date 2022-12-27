pub use crate::{
    ecs::Bundle,
    transform::{GlobalTransform, Transform},
};

use super::view::{ComputedVisibility, Visibility};

#[derive(Bundle, Debug, Default)]
pub struct SpatialBundle {
    /// The visibility of the entity.
    pub visibility: Visibility,
    /// The computed visibility of the entity.
    pub computed: ComputedVisibility,
    /// The transform of the entity.
    pub transform: Transform,
    /// The global transform of the entity.
    pub global_transform: GlobalTransform,
}

impl SpatialBundle {
    /// Creates a new [`SpatialBundle`] from a [`Transform`].
    ///
    /// This initializes [`GlobalTransform`] as identity, and visibility as visible
    #[inline]
    pub const fn from_transform(transform: Transform) -> Self {
        SpatialBundle {
            transform,
            ..Self::INHERITED_IDENTITY
        }
    }

    /// A visible [`SpatialBundle`], with no translation, rotation, and a scale of 1 on all axes.
    pub const INHERITED_IDENTITY: Self = SpatialBundle {
        visibility: Visibility::Inherited,
        computed: ComputedVisibility::HIDDEN,
        transform: Transform::IDENTITY,
        global_transform: GlobalTransform::IDENTITY,
    };

    /// An invisible [`SpatialBundle`], with no translation, rotation, and a scale of 1 on all axes.
    pub const HIDDEN_IDENTITY: Self = SpatialBundle {
        visibility: Visibility::Hidden,
        ..Self::INHERITED_IDENTITY
    };
}

impl From<Transform> for SpatialBundle {
    #[inline]
    fn from(transform: Transform) -> Self {
        Self::from_transform(transform)
    }
}
