mod children;
mod global_transform;
mod parent;
#[allow(clippy::module_inception)]
mod transform;
mod transform_propagate_system;

pub use children::Children;
pub use global_transform::GlobalTransform;
pub use parent::Parent;
pub use transform::Transform;

use crate::{
    app::{App, MainStage},
    ecs::Bundle,
};
use transform_propagate_system::transform_propagate_system;

#[derive(Bundle, Clone, Copy, Debug, Default)]
pub struct TransformBundle {
    pub local: Transform,
    pub global: GlobalTransform,
}

impl TransformBundle {
    #[inline]
    pub const fn from_transform(transform: Transform) -> Self {
        TransformBundle {
            local: transform,
            global: GlobalTransform::identity(),
        }
    }

    #[inline]
    pub const fn identity() -> Self {
        TransformBundle {
            local: Transform::identity(),
            global: GlobalTransform::identity(),
        }
    }
}

impl From<Transform> for TransformBundle {
    #[inline]
    fn from(transform: Transform) -> Self {
        Self::from_transform(transform)
    }
}

pub fn transform_plugin(app: &mut App) {
    app.add_system_to_stage(MainStage::PostUpdate, &transform_propagate_system);
}
