use std::marker::PhantomData;

use crate::ecs::{
    component::ComponentId,
    query::access::{Access, FilteredAccess},
    resource::ResourceId,
    World,
};

use super::{
    param::{SystemParam, SystemParamFetch, SystemParamState},
    IntoSystem, System, SystemId,
};

pub struct SystemMeta {
    pub(crate) id: SystemId,
    pub(crate) name: &'static str,
    pub(crate) resource_access: Access<ResourceId>,
    pub(crate) component_access: FilteredAccess<ComponentId>,
}

impl SystemMeta {
    fn new<T>(id: SystemId) -> Self {
        Self {
            id,
            name: std::any::type_name::<T>(),
            resource_access: Default::default(),
            component_access: Default::default(),
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

impl<In, Out, Param, Marker, F> IntoSystem<FunctionSystem<In, Out, Param, Marker, F>> for F
where
    In: 'static,
    Out: 'static,
    Param: SystemParam + 'static,
    Marker: 'static,
    F: SystemParamFunction<In, Out, Param, Marker>,
{
    fn system(self, id: SystemId, world: &mut World) -> FunctionSystem<In, Out, Param, Marker, F> {
        let mut meta = SystemMeta::new::<F>(id);
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
    fn name(&self) -> &'static str {
        self.system_meta.name
    }

    #[inline]
    fn id(&self) -> SystemId {
        self.system_meta.id
    }

    #[inline]
    fn resource_access(&self) -> &Access<ResourceId> {
        &self.system_meta.resource_access
    }

    #[inline]
    fn component_access(&self) -> &FilteredAccess<ComponentId> {
        &self.system_meta.component_access
    }

    #[inline]
    unsafe fn run(&mut self, input: Self::In, world: &World) -> Self::Out {
        self.param_state.update(world, &mut self.system_meta);
        self.func
            .run(input, &mut self.param_state, &self.system_meta, world)
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
            unsafe fn run(&mut self, _input: (), state: &mut <($($param,)*) as SystemParam>::Fetch, system_meta: &SystemMeta, world: &World) -> Out {
                let ($($param,)*) = <<($($param,)*) as SystemParam>::Fetch as SystemParamFetch>::get_param(state, system_meta, world);
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
            unsafe fn run(&mut self, input: Input, state: &mut <($($param,)*) as SystemParam>::Fetch, system_meta: &SystemMeta, world: &World) -> Out {
                let ($($param,)*) = <<($($param,)*) as SystemParam>::Fetch as SystemParamFetch>::get_param(state, system_meta, world);
                self(In(input), $($param),*)
            }
        }
    };
}

all_tuples!(impl_system_function);
