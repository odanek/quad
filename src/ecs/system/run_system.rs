use crate::ecs::{
    component::Tick, query::access::Access, ComponentId, ResourceId, SystemParam, SystemParamItem,
    World,
};

use super::{
    function_system::SystemMeta,
    system_param::{SystemParamFetch, SystemParamState},
    System,
};

pub trait RunSystem: Send + Sync + 'static {
    type Param: SystemParam;

    fn run(param: SystemParamItem<Self::Param>);

    fn system(world: &mut World) -> ParamSystem<Self::Param> {
        ParamSystem {
            run: Self::run,
            state: SystemState::new(world),
        }
    }
}

pub struct ParamSystem<P: SystemParam> {
    state: SystemState<P>,
    run: fn(SystemParamItem<P>),
}

impl<P: SystemParam + 'static> System for ParamSystem<P> {
    type In = ();

    type Out = ();

    fn name(&self) -> &str {
        &self.state.meta().name
    }

    #[inline]
    fn resource_access(&self) -> &Access<ResourceId> {
        &self.state.meta().resource_access
    }

    fn component_access(&self) -> &Access<ComponentId> {
        self.state.meta().component_access.combined_access()
    }

    unsafe fn run(&mut self, _input: Self::In, world: &World) -> Self::Out {
        let change_tick = world.increment_change_tick();
        self.state.update(world);
        let param = self.state.get_unchecked_manual(world, change_tick);
        (self.run)(param);
        self.state.advance_tick(change_tick);
    }

    fn apply_buffers(&mut self, world: &mut World) {
        self.state.apply(world);
    }
}

pub struct SystemState<Param: SystemParam> {
    meta: SystemMeta,
    param_state: <Param as SystemParam>::Fetch,
}

impl<Param: SystemParam> SystemState<Param> {
    pub fn new(world: &mut World) -> Self {
        let mut meta = SystemMeta::new(std::any::type_name::<Param>().to_owned());
        let param_state = <Param::Fetch as SystemParamState>::new(world, &mut meta);
        Self { meta, param_state }
    }

    #[inline]
    pub fn meta(&self) -> &SystemMeta {
        &self.meta
    }

    pub fn apply(&mut self, world: &mut World) {
        self.param_state.apply(world);
    }

    fn update(&mut self, world: &World) {
        self.param_state.update(world, &mut self.meta);
    }

    unsafe fn get_unchecked_manual<'w, 's>(
        &'s mut self,
        world: &'w World,
        change_tick: Tick,
    ) -> <Param::Fetch as SystemParamFetch<'w, 's>>::Item {
        <Param::Fetch as SystemParamFetch>::get_param(
            &mut self.param_state,
            &self.meta,
            world,
            change_tick,
        )
    }

    fn advance_tick(&mut self, change_tick: Tick) {
        self.meta.last_change_tick = change_tick;
    }
}
