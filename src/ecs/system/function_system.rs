use std::marker::PhantomData;

use crate::ecs::{
    archetype::{ArchetypeGeneration, ArchetypeId},
    query::access::Access,
    resource::ResourceId,
    world::WorldId,
    World,
};

use super::{
    system_param::{ReadOnlySystemParamFetch, SystemParam, SystemParamFetch, SystemParamState},
    IntoSystem, System, SystemId,
};

pub struct SystemMeta {
    pub(crate) id: SystemId,
    pub(crate) name: &'static str,
    pub(crate) resource_access: Access<ResourceId>,
    // pub(crate) component_access_set: FilteredAccessSet<ComponentId>,
    // pub(crate) archetype_component_access: Access<ArchetypeComponentId>,
}

impl SystemMeta {
    fn new<T>(id: SystemId) -> Self {
        Self {
            id,
            name: std::any::type_name::<T>(),
            resource_access: Default::default(),
            // archetype_component_access: Access::default(),
        }
    }
}

// pub struct SystemState<Param: SystemParam> {
//     meta: SystemMeta,
//     param_state: <Param as SystemParam>::Fetch,
//     world_id: WorldId,
//     archetype_generation: ArchetypeGeneration,
// }

// impl<Param: SystemParam> SystemState<Param> {
//     pub fn new(world: &mut World) -> Self {
//         let config = <Param::Fetch as SystemParamState>::default_config();
//         Self::with_config(world, config)
//     }

//     pub fn with_config(
//         world: &mut World,
//         config: <Param::Fetch as SystemParamState>::Config,
//     ) -> Self {
//         let mut meta = SystemMeta::new::<Param>();
//         let param_state = <Param::Fetch as SystemParamState>::init(world, &mut meta, config);
//         Self {
//             meta,
//             param_state,
//             world_id: world.id(),
//             archetype_generation: ArchetypeGeneration::initial(),
//         }
//     }

//     #[inline]
//     pub fn meta(&self) -> &SystemMeta {
//         &self.meta
//     }

//     /// Retrieve the [`SystemParam`] values. This can only be called when all parameters are read-only.
//     #[inline]
//     pub fn get<'a>(&'a mut self, world: &'a World) -> <Param::Fetch as SystemParamFetch<'a>>::Item
//     where
//         Param::Fetch: ReadOnlySystemParamFetch,
//     {
//         self.validate_world_and_update_archetypes(world);
//         unsafe { self.get_unchecked_manual(world) }
//     }

//     #[inline]
//     pub fn get_mut<'a>(
//         &'a mut self,
//         world: &'a mut World,
//     ) -> <Param::Fetch as SystemParamFetch<'a>>::Item {
//         self.validate_world_and_update_archetypes(world);
//         unsafe { self.get_unchecked_manual(world) }
//     }

//     pub fn apply(&mut self, world: &mut World) {
//         self.param_state.apply(world);
//     }

//     #[inline]
//     pub fn matches_world(&self, world: &World) -> bool {
//         self.world_id == world.id()
//     }

//     fn validate_world_and_update_archetypes(&mut self, world: &World) {
//         assert!(self.matches_world(world), "Encountered a mismatched World. A SystemState cannot be used with Worlds other than the one it was created with.");
//         let archetypes = world.archetypes();
//         let new_generation = archetypes.generation();
//         let old_generation = std::mem::replace(&mut self.archetype_generation, new_generation);
//         let archetype_index_range = old_generation.value()..new_generation.value();

//         for archetype_index in archetype_index_range {
//             self.param_state.new_archetype(
//                 &archetypes[ArchetypeId::new(archetype_index)],
//                 &mut self.meta,
//             );
//         }
//     }

//     #[inline]
//     pub unsafe fn get_unchecked_manual<'a>(
//         &'a mut self,
//         world: &'a World,
//     ) -> <Param::Fetch as SystemParamFetch<'a>>::Item {
//         <Param::Fetch as SystemParamFetch>::get_param(&mut self.param_state, &self.meta, world)
//     }
// }

pub struct In<In>(pub In);

pub struct InputMarker;

pub struct FunctionSystem<In, Out, Param, Marker, F>
where
    Param: SystemParam,
{
    func: F,
    param_state: Option<Param::Fetch>,
    system_meta: SystemMeta,
    config: Option<<Param::Fetch as SystemParamState>::Config>,

    // NOTE: PhantomData<fn()-> T> gives this safe Send/Sync impls
    #[allow(clippy::type_complexity)]
    marker: PhantomData<fn() -> (In, Out, Marker)>,
}

impl<In, Out, Param: SystemParam, Marker, F> FunctionSystem<In, Out, Param, Marker, F> {
    pub fn config(
        mut self,
        f: impl FnOnce(&mut <Param::Fetch as SystemParamState>::Config),
    ) -> Self {
        f(self.config.as_mut().unwrap());
        self
    }
}

impl<In, Out, Param, Marker, F> IntoSystem<FunctionSystem<In, Out, Param, Marker, F>> for F
where
    In: 'static,
    Out: 'static,
    Param: SystemParam + 'static,
    Marker: 'static,
    F: SystemParamFunction<In, Out, Param, Marker> + Send + Sync + 'static,
{
    fn system(self, id: SystemId) -> FunctionSystem<In, Out, Param, Marker, F> {
        FunctionSystem {
            func: self,
            param_state: None,
            config: Some(<Param::Fetch as SystemParamState>::default_config()),
            system_meta: SystemMeta::new::<F>(id),
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

    // #[inline]
    // fn new_archetype(&mut self, archetype: &Archetype) {
    //     let param_state = self.param_state.as_mut().unwrap();
    //     param_state.new_archetype(archetype, &mut self.system_meta);
    // }

    // #[inline]
    // fn component_access(&self) -> &Access<ComponentId> {
    //     &self.system_meta.component_access_set.combined_access()
    // }

    // #[inline]
    // fn archetype_component_access(&self) -> &Access<ArchetypeComponentId> {
    //     &self.system_meta.archetype_component_access
    // }

    #[inline]
    unsafe fn run_unsafe(&mut self, input: Self::In, world: &World) -> Self::Out {
        let out = self.func.run(
            input,
            self.param_state.as_mut().unwrap(),
            &self.system_meta,
            world,
        );
        out
    }

    #[inline]
    fn apply_buffers(&mut self, world: &mut World) {
        let param_state = self.param_state.as_mut().unwrap();
        param_state.apply(world);
    }

    #[inline]
    fn initialize(&mut self, world: &mut World) {
        self.param_state = Some(<Param::Fetch as SystemParamState>::init(
            world,
            &mut self.system_meta,
            self.config.take().unwrap(),
        ));
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
