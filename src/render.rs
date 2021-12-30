// mod camera;
// mod color;
// mod mesh;
// mod options;
// mod primitives;
// mod render_asset;
// mod render_component;
// mod render_graph;
// mod render_phase;
// mod render_resource;
// mod renderer;
// mod texture;
// mod view;

use std::ops::{Deref, DerefMut};

use crate::{app::App, ecs::World};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderStage {
    Extract,
    Prepare,
    Queue,
    PhaseSort,
    Render,
    Cleanup,
}

#[derive(Default)]
pub struct RenderWorld(World);

impl Deref for RenderWorld {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderWorld {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
struct ScratchRenderWorld(World);

fn render_plugin(app: &mut App, render_app: &mut App) {
    let options = app
        .world
        .get_resource::<options::WgpuOptions>()
        .cloned()
        .unwrap_or_default();
    let instance = wgpu::Instance::new(options.backends);
    let surface = {
        let world = app.world.cell();
        let windows = world.get_resource_mut::<bevy_window::Windows>().unwrap();
        let raw_handle = windows.get_primary().map(|window| unsafe {
            let handle = window.raw_window_handle().get_handle();
            instance.create_surface(&handle)
        });
        raw_handle
    };
    let (device, queue) = futures_lite::future::block_on(renderer::initialize_renderer(
        &instance,
        &wgpu::RequestAdapterOptions {
            power_preference: options.power_preference,
            compatible_surface: surface.as_ref(),
            ..Default::default()
        },
        &wgpu::DeviceDescriptor {
            label: options.device_label.as_ref().map(|a| a.as_ref()),
            features: options.features,
            limits: options.limits,
        },
    ));

    app.insert_resource(device.clone())
        .insert_resource(queue.clone())
        .add_asset::<Shader>()
        .init_asset_loader::<ShaderLoader>()
        .init_resource::<ScratchRenderWorld>()
        .register_type::<Color>()
        .register_type::<Frustum>();
    let render_pipeline_cache = RenderPipelineCache::new(device.clone());
    let asset_server = app.world.get_resource::<AssetServer>().unwrap().clone();

    let mut render_app = App::empty();
    let mut extract_stage =
        SystemStage::parallel().with_system(RenderPipelineCache::extract_shaders);
    // don't apply buffers when the stage finishes running
    // extract stage runs on the app world, but the buffers are applied to the render world
    extract_stage.set_apply_buffers(false);
    render_app
        .add_stage(RenderStage::Extract, extract_stage)
        .add_stage(RenderStage::Prepare, SystemStage::parallel())
        .add_stage(RenderStage::Queue, SystemStage::parallel())
        .add_stage(RenderStage::PhaseSort, SystemStage::parallel())
        .add_stage(
            RenderStage::Render,
            SystemStage::parallel()
                .with_system(RenderPipelineCache::process_pipeline_queue_system)
                .with_system(render_system.exclusive_system().at_end()),
        )
        .add_stage(RenderStage::Cleanup, SystemStage::parallel())
        .insert_resource(instance)
        .insert_resource(device)
        .insert_resource(queue)
        .insert_resource(render_pipeline_cache)
        .insert_resource(asset_server)
        .init_resource::<RenderGraph>();

    app.add_plugin(WindowRenderPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(ViewPlugin)
        .add_plugin(MeshPlugin)
        .add_plugin(ImagePlugin);
}

fn render_plugin_update(world: &mut World, render_app: &mut App) {
    let meta_len = app_world.entities().meta.len();
    render_app
        .world
        .entities()
        .reserve_entities(meta_len as u32);
    render_app.world.entities_mut().flush_as_invalid();

    extract(app_world, render_app);

    let prepare = render_app
        .schedule
        .get_stage_mut::<SystemStage>(&RenderStage::Prepare)
        .unwrap();
    prepare.run(&mut render_app.world);

    let queue = render_app
        .schedule
        .get_stage_mut::<SystemStage>(&RenderStage::Queue)
        .unwrap();
    queue.run(&mut render_app.world);

    let phase_sort = render_app
        .schedule
        .get_stage_mut::<SystemStage>(&RenderStage::PhaseSort)
        .unwrap();
    phase_sort.run(&mut render_app.world);

    let render = render_app
        .schedule
        .get_stage_mut::<SystemStage>(&RenderStage::Render)
        .unwrap();
    render.run(&mut render_app.world);

    let cleanup = render_app
        .schedule
        .get_stage_mut::<SystemStage>(&RenderStage::Cleanup)
        .unwrap();
    cleanup.run(&mut render_app.world);

    render_app.world.clear_entities();
}

fn extract(app_world: &mut World, render_app: &mut App) {
    let extract = render_app
        .schedule
        .get_stage_mut::<SystemStage>(&RenderStage::Extract)
        .unwrap();

    // temporarily add the render world to the app world as a resource
    let scratch_world = app_world.remove_resource::<ScratchRenderWorld>().unwrap();
    let render_world = std::mem::replace(&mut render_app.world, scratch_world.0);
    app_world.insert_resource(RenderWorld(render_world));

    extract.run(app_world);

    // add the render world back to the render app
    let render_world = app_world.remove_resource::<RenderWorld>().unwrap();
    let scratch_world = std::mem::replace(&mut render_app.world, render_world.0);
    app_world.insert_resource(ScratchRenderWorld(scratch_world));

    extract.apply_buffers(&mut render_app.world);
}
