mod context;
mod runner;
mod scene;
mod systems;
mod task_pool_options;

pub use scene::{Scene, SceneContext, SceneResult};
pub use systems::Stage;
pub use task_pool_options::DefaultTaskPoolOptions;

use crate::{
    asset::{asset_plugin, update_asset_storage_system, Asset, AssetEvent, AssetServer, Assets},
    ecs::{Event, Events, FromWorld, IntoSystem, Res, Resource, World},
    input::{input_plugin, KeyboardInput, MouseInput, Touches},
    timing::{timing_plugin, Time},
    windowing::{windowing_plugin, WindowBuilder, WindowId, Windows},
};

use self::{context::AppContext, runner::winit_runner, systems::Systems};

struct MainWindow(WindowBuilder);

impl Resource for MainWindow {}

#[derive(Default)]
pub struct App {
    world: World,
    systems: Systems,
}

impl App {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_default_pools(&mut self) -> &mut Self {
        let options = DefaultTaskPoolOptions::default();
        options.create_default_pools(&mut self.world);
        self
    }

    pub fn add_default_plugins(&mut self) -> &mut Self {
        timing_plugin(self);
        windowing_plugin(self);
        input_plugin(self);
        asset_plugin(self);
        self
    }

    pub fn init_resource<T: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<T>();
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
        self.insert_resource(MainWindow(window));
        self
    }

    pub fn run(&mut self, scene: Box<dyn Scene>) {
        let mut app = std::mem::take(self);
        let event_loop = winit::event_loop::EventLoop::new();
        let main_window_builder = app.world.remove_resource::<MainWindow>().unwrap().0;
        let main_window = main_window_builder.build(WindowId::new(0), &event_loop);
        let mut windows = app.world.resource_mut::<Windows>();
        windows.add(main_window);
        let context = AppContext::new(app, scene);
        winit_runner(context, event_loop);
    }

    pub fn update(&mut self) {
        self.before_update();
        self.after_update();
    }

    pub(crate) fn before_update(&mut self) {
        self.world.resource_mut::<Time>().update();
        self.systems.run(Stage::LoadAssets, &mut self.world);
        self.systems.run(Stage::PreUpdate, &mut self.world);
    }

    pub(crate) fn after_update(&mut self) {
        self.systems.run(Stage::PostUpdate, &mut self.world);
        self.systems.run(Stage::AssetEvents, &mut self.world);

        // Todo: Add as systems in input plugin
        self.world.resource_mut::<KeyboardInput>().flush();
        self.world.resource_mut::<MouseInput>().flush();
        self.world.resource_mut::<Touches>().flush();

        self.world.clear_trackers();
    }
}
