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

pub use quad_macros::{Component, Event, Resource, Bundle};

pub use component::{Bundle, Component, Components, ComponentId, DetectChanges, Res, ResMut, Resource, ResourceId};
pub use entity::Entity;
pub use event::{Event, EventId, EventReader, EventWriter, Events};
pub use query::{
    fetch::{ChangeTrackers, QueryItem, WorldQuery},
    filter::{Added, Changed, FilterFetch, Or, With, Without},
    state::QueryState,
};
pub use schedule::{OptionalSchedule, OptionalScheduleWithInput, Schedule, Scheduler};
pub use system::{
    command::Commands,
    local::Local,
    query::{Query, QuerySet},
    removed_components::RemovedComponents,
    run_system::RunSystem,
    system_param::{SystemParam, SystemParamItem},
    IntoSystem, System,
};
pub use world::{FromWorld, World};
