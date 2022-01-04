mod systems;
mod task_pool_options;

pub use systems::Stage;
pub use task_pool_options::TaskPoolOptions;

use crate::{
    asset::{
        asset_plugin, update_asset_storage_system, Asset, AssetEvent, AssetServer,
        AssetServerSettings, Assets,
    },
    ecs::{Event, Events, FromWorld, IntoSystem, Res, ResMut, Resource, World},
    input::input_plugin,
    render::render_plugin,
    timing::{timing_plugin, Time},
    windowing::{windowing_plugin, Window, Windows},
};

use self::systems::Systems;

#[derive(Default)]
pub struct App {
    pub(crate) world: World,     // TODO: Private?
    pub(crate) systems: Systems, // TODO: Private?
}

impl App {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn create_pools(&mut self, options: &TaskPoolOptions) {
        options.create_pools(&mut self.world);
    }

    pub fn add_timing_plugin(&mut self) -> &mut Self {
        timing_plugin(self);
        self
    }

    pub fn add_windowing_plugin(&mut self) -> &mut Self {
        windowing_plugin(self);
        self
    }

    pub fn add_input_plugin(&mut self) -> &mut Self {
        input_plugin(self);
        self
    }

    pub fn add_asset_plugin(&mut self, settings: &AssetServerSettings) -> &mut Self {
        self.world.insert_resource(settings.clone());
        asset_plugin(self);
        self
    }

    pub fn add_render_plugin(&mut self, render_app: &mut App) -> &mut Self {
        render_plugin(self, render_app);
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

    #[inline]
    pub fn get_resource<T: Resource>(&self) -> Option<Res<T>> {
        self.world.get_resource()
    }

    #[inline]
    pub fn resource<T: Resource>(&self) -> Res<T> {
        self.world.resource()
    }

    #[inline]
    pub fn resource_mut<T: Resource>(&mut self) -> ResMut<T> {
        self.world.resource_mut()
    }

    pub fn add_system_to_stage<S, Params>(&mut self, stage: Stage, system: S) -> &mut Self
    where
        S: IntoSystem<(), (), Params>,
    {
        self.systems.add(stage, system.system(&mut self.world));
        self
    }

    // TODO: AddEvent trait ?
    pub fn add_event<T: Event>(&mut self) -> &mut Self {
        self.init_resource::<Events<T>>().add_system_to_stage(
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
            .add_system_to_stage(Stage::AssetEvents, &Assets::<T>::asset_event_system)
            .add_system_to_stage(Stage::LoadAssets, &update_asset_storage_system::<T>)
            .add_event::<AssetEvent<T>>();

        self
    }

    // TODO: AddWindow trait
    pub(crate) fn add_window(&mut self, window: Window) -> &mut Self {
        let mut windows = self.world.resource_mut::<Windows>();
        windows.add(window);
        self
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
        self.systems.run(Stage::Flush, &mut self.world);
        self.world.clear_trackers();
    }
}
