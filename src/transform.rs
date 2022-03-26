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

use crate::app::{App, Stage};
use transform_propagate_system::transform_propagate_system;

pub fn transform_plugin(app: &mut App) {
    app.add_system_to_stage(Stage::PostUpdate, &transform_propagate_system);
}
