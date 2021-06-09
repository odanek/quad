use crate::ecs::{World, archetype::Archetype};

use super::function_system::SystemMeta;

pub trait SystemParam: Sized {
    type Fetch: for<'a> SystemParamFetch<'a>;
}

pub unsafe trait SystemParamState: Send + Sync + 'static {
    type Config: Send + Sync;

    fn init(world: &mut World, system_meta: &mut SystemMeta, config: Self::Config) -> Self;
    
    #[inline]
    fn new_archetype(&mut self, _archetype: &Archetype, _system_meta: &mut SystemMeta) {}
    
    #[inline]
    fn apply(&mut self, _world: &mut World) {}
    
    fn default_config() -> Self::Config;
}

pub unsafe trait ReadOnlySystemParamFetch {}

pub trait SystemParamFetch<'a>: SystemParamState {
    type Item;

    unsafe fn get_param(
        state: &'a mut Self,
        system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item;
}