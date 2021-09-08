mod bundle;
mod change_detection;
#[allow(clippy::module_inception)]
mod component;
mod resource;
mod ticks;
mod type_info;

pub use bundle::{Bundle, BundleId, BundleInfo, Bundles};
pub use change_detection::{CmptMut, Res, ResMut, Ticks};
pub use component::{Component, ComponentId, ComponentInfo, ComponentStatus, Components};
pub use resource::{Resource, ResourceId, Resources};
pub use ticks::{ComponentTicks, Tick};
