pub mod cameras;
pub mod color;
pub mod primitives;
pub mod render_asset;
pub mod render_component;
pub mod render_graph;
pub mod render_phase;
pub mod render_resource;
pub mod renderer;
pub mod settings;
pub mod texture;
pub mod view;

use derive_deref::{Deref, DerefMut};

use crate::{
    app::{App, RenderStage},
    asset::AssetServer,
    ecs::{IntoSystem, Resource, World},
    render::{
        cameras::camera_plugin,
        render_graph::RenderGraph,
        render_resource::RenderPipelineCache,
        renderer::{render_system, RenderInstance},
        texture::image_plugin,
        view::{view_plugin, window::window_render_plugin},
    },
    windowing::Windows,
};

use self::render_resource::{Shader, ShaderLoader};

pub use wgpu::AddressMode;

pub mod prelude {
    pub use crate::render::{
        cameras::Camera2d, color::Color, texture::Image, view::Visibility, AddressMode,
    };
}

/// The Render App World. This is only available as a resource during the Extract step.
#[derive(Default, Resource, Deref, DerefMut)]
pub struct RenderWorld(World);

/// A Label for the rendering sub-app.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RenderApp;

/// A "scratch" world used to avoid allocating new worlds every frame when
/// swapping out the [`RenderWorld`].
#[derive(Default, Resource)]
struct ScratchRenderWorld(World);

// TODO Maybe this should create the render app and return it
pub fn render_plugin(app: &mut App, render_app: &mut App) {
    let options = app
        .world
        .get_resource::<settings::WgpuSettings>()
        .as_deref()
        .cloned()
        .unwrap_or_default();

    app.add_asset::<Shader>()
        .init_asset_loader::<ShaderLoader>();

    let instance = wgpu::Instance::new(options.backends);
    let surface = {
        let windows = app.world.resource_mut::<Windows>();
        let raw_handle = windows.get_primary().map(|window| unsafe {
            let handle = window.raw_window_handle().get_handle();
            instance.create_surface(&handle)
        });
        raw_handle
    };
    let request_adapter_options = wgpu::RequestAdapterOptions {
        power_preference: options.power_preference,
        compatible_surface: surface.as_ref(),
        ..Default::default()
    };
    let (device, queue, adapter_info) = futures_lite::future::block_on(
        renderer::initialize_renderer(&instance, &options, &request_adapter_options),
    );
    log::debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
    log::debug!("Configured wgpu adapter Features: {:#?}", device.features());
    app.insert_resource(device.clone())
        .insert_resource(queue.clone())
        .insert_resource(adapter_info.clone())
        .init_resource::<ScratchRenderWorld>();

    let render_pipeline_cache = RenderPipelineCache::new(device.clone());
    let asset_server = app.world.resource::<AssetServer>().clone();

    render_app
        .insert_resource(RenderInstance::new(instance))
        .insert_resource(device)
        .insert_resource(queue)
        .insert_resource(adapter_info)
        .insert_resource(render_pipeline_cache)
        .insert_resource(asset_server)
        .init_resource::<RenderGraph>();

    // Has to use world from the main app
    render_app.add_system_to_stage(
        RenderStage::Extract,
        (RenderPipelineCache::extract_shaders).system(&mut app.world),
    );

    render_app.add_system_to_stage(
        RenderStage::Render,
        RenderPipelineCache::process_pipeline_queue_system,
    );

    window_render_plugin(app, render_app);
    camera_plugin(app, render_app);
    view_plugin(app, render_app);
    image_plugin(app, render_app);
}

pub fn update_render_app(app_world: &mut World, render_app: &mut App) {
    // reserve all existing app entities for use in render_app
    // they can only be spawned using `get_or_spawn()`
    let meta_len = app_world.entities().meta_len();
    render_app
        .world
        .entities()
        .reserve_entities(meta_len as u32);

    // flushing as "invalid" ensures that app world entities aren't added as "empty archetype" entities by default
    // these entities cannot be accessed without spawning directly onto them
    // this _only_ works as expected because clear_entities() is called at the end of every frame.
    render_app.world.entities_mut().flush_as_invalid();

    // extract
    extract(app_world, render_app);

    // prepare
    render_app
        .systems
        .run(RenderStage::Prepare, &mut render_app.world);

    // queue
    render_app
        .systems
        .run(RenderStage::Queue, &mut render_app.world);

    // phase sort
    render_app
        .systems
        .run(RenderStage::PhaseSort, &mut render_app.world);

    // render
    render_app
        .systems
        .run(RenderStage::Render, &mut render_app.world);

    render_system(&mut render_app.world);

    // cleanup
    render_app
        .systems
        .run(RenderStage::Cleanup, &mut render_app.world);

    render_app.world.clear_entities();
}

/// Executes the [`Extract`](RenderStage::Extract) stage of the renderer.
/// This updates the render world with the extracted ECS data of the current frame.
fn extract(app_world: &mut World, render_app: &mut App) {
    // temporarily add the render world to the app world as a resource
    let scratch_world = app_world.remove_resource::<ScratchRenderWorld>().unwrap();
    let render_world = std::mem::replace(&mut render_app.world, scratch_world.0);
    app_world.insert_resource(RenderWorld(render_world));

    render_app
        .systems
        .get(RenderStage::Extract)
        .unwrap()
        .run(app_world);

    // add the render world back to the render app
    let render_world = app_world.remove_resource::<RenderWorld>().unwrap();
    let scratch_world = std::mem::replace(&mut render_app.world, render_world.0);
    app_world.insert_resource(ScratchRenderWorld(scratch_world));

    render_app
        .systems
        .get(RenderStage::Extract)
        .unwrap()
        .apply_buffers(&mut render_app.world);
}
