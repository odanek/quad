mod component;
mod entity;
mod event;
mod query;
mod schedule;
mod storage;
mod system;
mod world;

pub use quad_macros::{Bundle, Component, Event, Resource};

pub use component::{
    Bundle, Component, ComponentId, Components, DetectChanges, Res, ResMut, Resource, ResourceId,
};
pub use entity::Entity;
pub use event::{Event, EventId, EventReader, EventWriter, Events};
pub use query::{
    fetch::{ChangeTrackers, QueryItem, WorldQuery},
    filter::{Added, Changed, FilterFetch, Or, With, Without},
    state::QueryState,
};
pub use schedule::{Schedule, Scheduler};
pub use system::{
    command::Commands,
    local::Local,
    query::Query,
    removed_components::RemovedComponents,
    system_param::{
        ParamSet, ReadOnlySystemParamFetch, StaticSystemParam, SystemParam, SystemParamItem,
        SystemState,
    },
    IntoSystem, System,
};
pub use world::{FromWorld, World};

pub mod prelude {
    pub use crate::ecs::{
        Added, Bundle, ChangeTrackers, Changed, Commands, Component, DetectChanges, Entity,
        EventReader, EventWriter, FromWorld, IntoSystem, Local, Or, ParamSet, Query, QueryState,
        RemovedComponents, Res, ResMut, Resource, Schedule, Scheduler, System, With, Without,
        World,
    };
}
