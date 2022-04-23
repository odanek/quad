use std::{any::type_name, marker::PhantomData};

use crate::{
    ecs::{
        component::{ComponentId, ResourceId, Tick},
        query::access::{Access, FilteredAccessSet},
        World,
    },
    macros::all_tuples,
};

use super::{
    system_param::{SystemParam, SystemParamFetch, SystemParamState},
    IntoSystem, System,
};

#[derive(Clone)]
pub struct SystemMeta {
    pub(crate) name: String,
    pub(crate) resource_access: Access<ResourceId>,
    pub(crate) component_access: FilteredAccessSet<ComponentId>,
    pub(crate) last_change_tick: Tick,
}

impl SystemMeta {
    pub fn new(name: String) -> Self {
        Self {
            name,
            resource_access: Default::default(),
            component_access: Default::default(),
            last_change_tick: Default::default(),
        }
    }
}

pub struct In<In>(pub In);

pub struct InputMarker;

pub struct FunctionSystem<In, Out, Param, Marker, F>
where
    Param: SystemParam,
{
    func: F,
    param_state: Param::Fetch,
    system_meta: SystemMeta,

    // NOTE: PhantomData<fn()-> T> gives this safe Send/Sync impls
    #[allow(clippy::type_complexity)]
    marker: PhantomData<fn() -> (In, Out, Marker)>,
}

impl<In, Out, Param, Marker, F> IntoSystem<In, Out, (Param, Marker)> for F
where
    In: 'static,
    Out: 'static,
    Param: SystemParam + 'static,
    Marker: 'static,
    F: SystemParamFunction<In, Out, Param, Marker>,
{
    type System = FunctionSystem<In, Out, Param, Marker, F>;

    fn system(self, world: &mut World) -> Self::System {
        let mut meta = SystemMeta::new(type_name::<F>().to_owned());
        FunctionSystem {
            func: self,
            param_state: <Param::Fetch as SystemParamState>::new(world, &mut meta),
            system_meta: meta,
            marker: PhantomData,
        }
    }
}

impl<In, Out, Param, Marker, F> System for FunctionSystem<In, Out, Param, Marker, F>
where
    In: 'static,
    Out: 'static,
    Param: SystemParam + 'static,
    Marker: 'static,
    F: SystemParamFunction<In, Out, Param, Marker> + Send + Sync + 'static,
{
    type In = In;
    type Out = Out;

    #[inline]
    fn name(&self) -> &str {
        &self.system_meta.name
    }

    #[inline]
    fn resource_access(&self) -> &Access<ResourceId> {
        &self.system_meta.resource_access
    }

    #[inline]
    fn component_access(&self) -> &Access<ComponentId> {
        self.system_meta.component_access.combined_access()
    }

    #[inline]
    unsafe fn run(&mut self, input: Self::In, world: &World) -> Self::Out {
        let change_tick = world.increment_change_tick();
        self.param_state.update(world, &mut self.system_meta);
        let result = self.func.run(
            input,
            &mut self.param_state,
            &self.system_meta,
            world,
            change_tick,
        );
        self.system_meta.last_change_tick = change_tick;
        result
    }

    #[inline]
    fn apply_buffers(&mut self, world: &mut World) {
        self.param_state.apply(world);
    }
}

pub trait SystemParamFunction<In, Out, Param: SystemParam, Marker>: Send + Sync + 'static {
    unsafe fn run(
        &mut self,
        input: In,
        state: &mut Param::Fetch,
        system_meta: &SystemMeta,
        world: &World,
        change_tick: Tick,
    ) -> Out;
}

macro_rules! impl_system_function {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<Out, Func, $($param: SystemParam),*> SystemParamFunction<(), Out, ($($param,)*), ()> for Func
        where
            Func:
                FnMut($($param),*) -> Out +
                FnMut($(<<$param as SystemParam>::Fetch as SystemParamFetch>::Item),*) -> Out + Send + Sync + 'static, Out: 'static
        {
            #[inline]
            unsafe fn run(&mut self, _input: (), state: &mut <($($param,)*) as SystemParam>::Fetch, system_meta: &SystemMeta, world: &World, change_tick: Tick) -> Out {
                let ($($param,)*) = <<($($param,)*) as SystemParam>::Fetch as SystemParamFetch>::get_param(state, system_meta, world, change_tick);
                self($($param),*)
            }
        }

        #[allow(non_snake_case)]
        impl<Input, Out, Func, $($param: SystemParam),*> SystemParamFunction<Input, Out, ($($param,)*), InputMarker> for Func
        where
            Func:
                FnMut(In<Input>, $($param),*) -> Out +
                FnMut(In<Input>, $(<<$param as SystemParam>::Fetch as SystemParamFetch>::Item),*) -> Out + Send + Sync + 'static, Out: 'static
        {
            #[inline]
            unsafe fn run(&mut self, input: Input, state: &mut <($($param,)*) as SystemParam>::Fetch, system_meta: &SystemMeta, world: &World, change_tick: Tick) -> Out {
                let ($($param,)*) = <<($($param,)*) as SystemParam>::Fetch as SystemParamFetch>::get_param(state, system_meta, world, change_tick);
                self(In(input), $($param),*)
            }
        }
    };
}

all_tuples!(impl_system_function);
