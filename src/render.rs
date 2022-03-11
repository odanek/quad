mod cameras;
mod color;
// mod mesh;
mod options;
mod primitives;
mod render_asset;
mod render_component;
mod render_graph;
// mod render_phase;
mod render_resource;
mod renderer;
mod texture;
mod view;

use std::ops::{Deref, DerefMut};

use crate::{
    app::App,
    ecs::{Resource, World},
    windowing::Windows,
};

use self::{
    options::WgpuOptions,
    render_resource::{Shader, ShaderLoader},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderStage {
    Extract,
    Prepare,
    Queue,
    PhaseSort,
    Render,
    Cleanup,
}

// TODO: How to avoid the need for this and unsafe Send/Sync impl on Table and Resources?
#[derive(Default, Resource)]
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

pub fn render_plugin(app: &mut App, render_app: &mut App) {
    let mut options = app
        .get_resource::<WgpuOptions>()
        .as_deref()
        .cloned()
        .unwrap_or_default();

    app.add_asset::<Shader>()
        .init_asset_loader::<ShaderLoader>();

    let instance = wgpu::Instance::new(options.backends);
    let surface = {
        let windows = app.resource_mut::<Windows>();
        let raw_handle = windows
            .get_primary()
            .map(|window| unsafe { instance.create_surface(window.winit_window()) });
        raw_handle
    };

    let request_adapter_options = wgpu::RequestAdapterOptions {
        power_preference: options.power_preference,
        compatible_surface: surface.as_ref(),
        ..Default::default()
    };
    let (device, queue) = futures_lite::future::block_on(renderer::initialize_renderer(
        &instance,
        &mut options,
        &request_adapter_options,
    ));

    // debug!("Configured wgpu adapter Limits: {:#?}", &options.limits);
    // debug!("Configured wgpu adapter Features: {:#?}", &options.features);
    // app.insert_resource(device.clone())
    //     .insert_resource(queue.clone())
    //     .insert_resource(options.clone())
    //     .init_resource::<ScratchRenderWorld>()
    //     .register_type::<Frustum>()
    //     .register_type::<CubemapFrusta>();
    // let render_pipeline_cache = RenderPipelineCache::new(device.clone());
    // let asset_server = app.world.get_resource::<AssetServer>().unwrap().clone();

    // let mut render_app = App::empty();
    // let mut extract_stage =
    //     SystemStage::parallel().with_system(RenderPipelineCache::extract_shaders);
    // // don't apply buffers when the stage finishes running
    // // extract stage runs on the app world, but the buffers are applied to the render world
    // extract_stage.set_apply_buffers(false);
    // render_app
    //     .add_stage(RenderStage::Extract, extract_stage)
    //     .add_stage(RenderStage::Prepare, SystemStage::parallel())
    //     .add_stage(RenderStage::Queue, SystemStage::parallel())
    //     .add_stage(RenderStage::PhaseSort, SystemStage::parallel())
    //     .add_stage(
    //         RenderStage::Render,
    //         SystemStage::parallel()
    //             .with_system(RenderPipelineCache::process_pipeline_queue_system)
    //             .with_system(render_system.exclusive_system().at_end()),
    //     )
    //     .add_stage(RenderStage::Cleanup, SystemStage::parallel())
    //     .insert_resource(instance)
    //     .insert_resource(device)
    //     .insert_resource(queue)
    //     .insert_resource(options)
    //     .insert_resource(render_pipeline_cache)
    //     .insert_resource(asset_server)
    //     .init_resource::<RenderGraph>();
}

fn render_plugin_update(world: &mut World, render_app: &mut App) {
    // // reserve all existing app entities for use in render_app
    // // they can only be spawned using `get_or_spawn()`
    // let meta_len = app_world.entities().meta.len();
    // render_app
    //     .world
    //     .entities()
    //     .reserve_entities(meta_len as u32);

    // // flushing as "invalid" ensures that app world entities aren't added as "empty archetype" entities by default
    // // these entities cannot be accessed without spawning directly onto them
    // // this _only_ works as expected because clear_entities() is called at the end of every frame.
    // render_app.world.entities_mut().flush_as_invalid();

    // // extract
    // extract(app_world, render_app);

    // // prepare
    // let prepare = render_app
    //     .schedule
    //     .get_stage_mut::<SystemStage>(&RenderStage::Prepare)
    //     .unwrap();
    // prepare.run(&mut render_app.world);

    // // queue
    // let queue = render_app
    //     .schedule
    //     .get_stage_mut::<SystemStage>(&RenderStage::Queue)
    //     .unwrap();
    // queue.run(&mut render_app.world);

    // // phase sort
    // let phase_sort = render_app
    //     .schedule
    //     .get_stage_mut::<SystemStage>(&RenderStage::PhaseSort)
    //     .unwrap();
    // phase_sort.run(&mut render_app.world);

    // // render
    // let render = render_app
    //     .schedule
    //     .get_stage_mut::<SystemStage>(&RenderStage::Render)
    //     .unwrap();
    // render.run(&mut render_app.world);

    // // cleanup
    // let cleanup = render_app
    //     .schedule
    //     .get_stage_mut::<SystemStage>(&RenderStage::Cleanup)
    //     .unwrap();
    // cleanup.run(&mut render_app.world);

    // render_app.world.clear_entities();
}

// fn extract(app_world: &mut World, render_app: &mut App) {
//     let extract = render_app
//         .schedule
//         .get_stage_mut::<SystemStage>(&RenderStage::Extract)
//         .unwrap();

//     // temporarily add the render world to the app world as a resource
//     let scratch_world = app_world.remove_resource::<ScratchRenderWorld>().unwrap();
//     let render_world = std::mem::replace(&mut render_app.world, scratch_world.0);
//     app_world.insert_resource(RenderWorld(render_world));

//     extract.run(app_world);

//     // add the render world back to the render app
//     let render_world = app_world.remove_resource::<RenderWorld>().unwrap();
//     let scratch_world = std::mem::replace(&mut render_app.world, render_world.0);
//     app_world.insert_resource(ScratchRenderWorld(scratch_world));

//     extract.apply_buffers(&mut render_app.world);
// }
