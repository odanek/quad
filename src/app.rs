mod context;
mod runner;
mod scene;
mod systems;

pub use scene::{Scene, SceneContext, SceneResult};
pub use systems::Stage;

use crate::{
    asset::{asset_plugin, update_asset_storage_system, Asset, AssetEvent, AssetServer, Assets},
    ecs::{Event, Events, FromWorld, IntoSystem, Res, Resource, World},
    input::input_plugin,
    tasks::{logical_core_count, IoTaskPool, TaskPoolBuilder},
    timing::timing_plugin,
    windowing::{windowing_plugin, WindowBuilder, WindowId, Windows},
};

use self::{context::AppContext, runner::winit_runner, systems::Systems};

#[derive(Default)]
pub struct App {
    world: World,
    systems: Systems,
    main_window_builder: WindowBuilder,
}

impl App {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_default_plugins(&mut self) -> &mut Self {
        self.add_standard_pools(1, usize::MAX);

        timing_plugin(self);
        windowing_plugin(self);
        input_plugin(self);
        asset_plugin(self);
        self
    }

    pub fn init_resource<T: Resource + FromWorld>(&mut self) -> &mut Self {
        let resource = T::from_world(&mut self.world);
        self.insert_resource(resource);
        self
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn resource<T: Resource>(&self) -> Res<T> {
        self.world.resource()
    }

    pub fn add_system<S, Params>(&mut self, stage: Stage, system: S) -> &mut Self
    where
        S: IntoSystem<(), (), Params>,
    {
        self.systems.add(stage, system.system(&mut self.world));
        self
    }

    // TODO: AddEvent trait ?
    pub fn add_event<T: Event>(&mut self) -> &mut Self {
        self.init_resource::<Events<T>>().add_system(
            Stage::PreUpdate,
            &Events::<T>::update_system, // TODO: Why is the & needed?
        );

        self
    }

    // TODO: AddAsset trait
    pub fn add_asset<T: Asset>(&mut self) -> &mut Self {
        let assets = {
            let asset_server = self
                .world
                .get_resource::<AssetServer>()
                .expect("Asset plugin not initialized");
            asset_server.register_asset_type::<T>()
        };

        self.insert_resource(assets)
            .add_system(Stage::AssetEvents, &Assets::<T>::asset_event_system)
            .add_system(Stage::LoadAssets, &update_asset_storage_system::<T>)
            .add_event::<AssetEvent<T>>();

        self
    }

    pub fn main_window(&mut self, window: WindowBuilder) -> &mut Self {
        self.main_window_builder = window;
        self
    }

    pub fn run(&mut self, scene: Box<dyn Scene>) {
        let App {
            mut world,
            systems,
            main_window_builder,
        } = std::mem::take(self);
        let event_loop = winit::event_loop::EventLoop::new();
        let main_window = main_window_builder.build(WindowId::new(0), &event_loop);
        let mut windows = world.resource_mut::<Windows>();
        windows.add(main_window);
        let context = AppContext::new(world, systems, scene);
        winit_runner(context, event_loop);
    }

    fn add_standard_pools(&mut self, min_total_threads: usize, max_total_threads: usize) {
        let total_threads = logical_core_count().clamp(min_total_threads, max_total_threads);
        log::trace!("Assigning {} cores to default task pools", total_threads);

        let remaining_threads = total_threads;

        let io_threads = get_number_of_threads(remaining_threads, total_threads, 0.25, 1, 4);

        log::trace!("IO Threads: {}", io_threads);
        // remaining_threads = remaining_threads.saturating_sub(io_threads);

        self.world.insert_resource(IoTaskPool(
            TaskPoolBuilder::default()
                .num_threads(io_threads)
                .thread_name("IO Task Pool".to_string())
                .build(),
        ));
    }
}

fn get_number_of_threads(
    remaining_threads: usize,
    total_threads: usize,
    percent: f32,
    min_threads: usize,
    max_threads: usize,
) -> usize {
    let mut desired = (total_threads as f32 * percent).round() as usize;
    desired = desired.min(remaining_threads);
    desired.clamp(min_threads, max_threads)
}
