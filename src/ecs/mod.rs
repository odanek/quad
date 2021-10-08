#[macro_use]
mod macros;

mod component;
mod entity;
mod event;
mod query;
mod schedule;
mod storage;
mod system;
mod world;

pub use quad_macros::Component;
pub use quad_macros::Resource;

pub use component::{Component, ComponentId, Res, ResMut, Resource, ResourceId};
pub use entity::Entity;
pub use event::{Event, EventId, EventReader, EventWriter, Events};
pub use query::{
    fetch::ChangeTrackers,
    filter::{Added, Changed, Or, With, Without},
};
pub use schedule::{OptionalSchedule, OptionalScheduleWithInput, Schedule, Scheduler};
pub use system::{
    command::Commands, local::Local, query::Query, removed_components::RemovedComponents,
    IntoSystem, System,
};
pub use world::{FromWorld, World};
