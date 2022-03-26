mod systems;
mod task_pool_options;

pub use systems::Stage;
pub use task_pool_options::TaskPoolOptions;

use crate::{
    asset::{
        asset_plugin, update_asset_storage_system, Asset, AssetEvent, AssetLoader, AssetServer,
        AssetServerSettings, Assets,
    },
    ecs::{
        Event, Events, FromWorld, IntoSystem, ReadOnlySystemParamFetch, Res, ResMut, Resource,
        SystemParam, World,
    },
    input::input_plugin,
    pipeline::core_pipeline_plugin,
    render::{
        render_phase::{DrawFunctions, PhaseItem, RenderCommand, RenderCommandState},
        render_plugin, update_render_app,
    },
    sprite::sprite_plugin,
    timing::{timing_plugin, Time},
    transform::transform_plugin,
    windowing::{windowing_plugin, Window, Windows},
    Scene, SceneResult,
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

    pub fn add_transform_plugin(&mut self) -> &mut Self {
        transform_plugin(self);
        self
    }

    // TODO Move to MainApp trait
    pub(crate) fn add_render_plugin(&mut self, render_app: &mut App) -> &mut Self {
        render_plugin(self, render_app);
        core_pipeline_plugin(self, render_app);
        sprite_plugin(self, render_app);
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

    // TODO AddRenderCommand trait?
    pub(crate) fn add_render_command<P: PhaseItem, C: RenderCommand<P> + Send + Sync + 'static>(
        &mut self,
    ) -> &mut Self
    where
        <C::Param as SystemParam>::Fetch: ReadOnlySystemParamFetch,
    {
        let draw_function = RenderCommandState::<P, C>::new(&mut self.world);
        let draw_functions = self.world.resource::<DrawFunctions<P>>();
        draw_functions.write().add_with::<C, _>(draw_function);
        self
    }

    pub fn init_asset_loader<T: AssetLoader + FromWorld>(&mut self) -> &mut Self {
        let result = T::from_world(&mut self.world);
        self.add_asset_loader(result)
    }

    fn add_asset_loader<T: AssetLoader>(&mut self, loader: T) -> &mut Self {
        self.world.resource_mut::<AssetServer>().add_loader(loader);
        self
    }

    // TODO: AddWindow trait
    pub(crate) fn add_window(&mut self, window: Window) -> &mut Self {
        let mut windows = self.world.resource_mut::<Windows>();
        windows.add(window);
        self
    }
}

pub trait MainApp {
    fn update_main_app(&mut self, render_app: &mut App, scene: &mut dyn Scene) -> SceneResult;
}

impl MainApp for App {
    fn update_main_app(&mut self, render_app: &mut App, scene: &mut dyn Scene) -> SceneResult {
        self.world.resource_mut::<Time>().update();
        self.systems.run(Stage::LoadAssets, &mut self.world);
        self.systems.run(Stage::PreUpdate, &mut self.world);

        let result = scene.update(&mut self.world);
        if matches!(result, SceneResult::Quit) {
            return result;
        }

        self.systems.run(Stage::PostUpdate, &mut self.world);
        self.systems.run(Stage::AssetEvents, &mut self.world);
        self.systems.run(Stage::Flush, &mut self.world);
        self.world.clear_trackers();

        update_render_app(&mut self.world, render_app);

        result
    }
}
