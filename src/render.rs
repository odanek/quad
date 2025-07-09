pub mod cameras;
pub mod color;
pub mod extract_param;
pub mod primitives;
pub mod render_asset;
pub mod render_graph;
pub mod render_phase;
pub mod render_resource;
pub mod renderer;
pub mod settings;
pub mod spatial_bundle;
pub mod texture;
pub mod view;

use derive_deref::{Deref, DerefMut};

use crate::{
    app::{App, RenderStage},
    asset::AssetServer,
    ecs::{Resource, World},
    render::{
        cameras::camera_plugin,
        render_graph::RenderGraph,
        render_resource::RenderPipelineCache,
        renderer::{RenderInstance, render_system},
        texture::image_plugin,
        view::{view_plugin, window::window_render_plugin},
    },
    windowing::Windows,
};

use self::render_resource::{Shader, ShaderLoader};

pub use wgpu::AddressMode;

pub mod prelude {
    pub use crate::render::{
        AddressMode,
        cameras::Camera2d,
        color::Color,
        spatial_bundle::SpatialBundle,
        texture::Image,
        view::{Visibility, VisibilityBundle},
    };
}

#[derive(Default, Resource, Deref, DerefMut)]
pub struct MainWorld(World);

/// A Label for the rendering sub-app.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RenderApp;

/// A "scratch" world used to avoid allocating new worlds every frame when
/// swapping out the [`MainWorld`].
#[derive(Default, Resource)]
struct ScratchMainWorld(World);

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

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: options.backends,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });
    let surface = {
        let windows = app.world.resource_mut::<Windows>();
        windows.get_primary().map(|window| {
            let handle = window.window_handle();
            instance.create_surface(handle).unwrap()
        })
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
        .init_resource::<ScratchMainWorld>();

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

    render_app.add_system_to_stage(RenderStage::Extract, RenderPipelineCache::extract_shaders);

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
    // temporarily add the app world to the render world as a resource
    let scratch_world = app_world.remove_resource::<ScratchMainWorld>().unwrap();
    let inserted_world = std::mem::replace(app_world, scratch_world.0);
    let running_world = &mut render_app.world;
    running_world.insert_resource(MainWorld(inserted_world));

    render_app
        .systems
        .get(RenderStage::Extract)
        .unwrap()
        .run(running_world);

    // move the app world back, as if nothing happened.
    let inserted_world = running_world.remove_resource::<MainWorld>().unwrap();
    let scratch_world = std::mem::replace(app_world, inserted_world.0);
    app_world.insert_resource(ScratchMainWorld(scratch_world));

    render_app
        .systems
        .get(RenderStage::Extract)
        .unwrap()
        .apply_buffers(running_world);
}
