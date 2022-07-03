mod graph_runner;
mod render_device;

use derive_deref::{Deref, DerefMut};
pub use graph_runner::*;
pub use render_device::*;

use std::sync::Arc;
use wgpu::{AdapterInfo, CommandEncoder, Instance, Queue, RequestAdapterOptions};

use crate::ecs::{Entity, ResMut, Resource, With, World};

use super::{
    render_graph::RenderGraph,
    settings::WgpuSettings,
    view::{window::ExtractedWindows, ViewTarget},
};

/// Updates the [`RenderGraph`] with all of its nodes and then runs it to render the entire frame.
pub fn render_system(world: &mut World) {
    world.resource_scope(|world, mut graph: ResMut<RenderGraph>| {
        graph.update(world);
    });
    let graph = world.resource::<RenderGraph>();
    let render_device = world.resource::<RenderDevice>();
    let render_queue = world.resource::<RenderQueue>();

    if let Err(e) = RenderGraphRunner::run(
        graph.as_ref(),
        render_device.clone(), // TODO: is this clone really necessary?
        render_queue.as_ref(),
        world,
    ) {
        log::error!("Error running render graph:");
        {
            let mut src: &dyn std::error::Error = &e;
            loop {
                log::error!("> {}", src);
                match src.source() {
                    Some(s) => src = s,
                    None => break,
                }
            }
        }

        panic!("Error running render graph: {}", e);
    }

    // Remove ViewTarget components to ensure swap chain TextureViews are dropped.
    // If all TextureViews aren't dropped before present, acquiring the next swap chain texture will fail.
    let view_entities = world
        .query_filtered::<Entity, With<ViewTarget>>()
        .iter(world)
        .collect::<Vec<_>>();
    for view_entity in view_entities {
        world.entity_mut(view_entity).remove::<ViewTarget>();
    }

    let mut windows = world.resource_mut::<ExtractedWindows>();
    for window in windows.values_mut() {
        if let Some(texture_view) = window.swap_chain_texture.take() {
            if let Some(surface_texture) = texture_view.take_surface_texture() {
                surface_texture.present();
            }
        }
    }
}

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct RenderQueue(Arc<Queue>);

#[derive(Resource, Deref, DerefMut)]
pub struct RenderInstance(Instance);

impl RenderInstance {
    pub fn new(instance: Instance) -> Self {
        Self(instance)
    }
}

#[derive(Resource, Clone, Deref)]
pub struct RenderAdapterInfo(AdapterInfo);

/// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
/// for the specified backend.
pub async fn initialize_renderer(
    instance: &Instance,
    options: &WgpuSettings,
    request_adapter_options: &RequestAdapterOptions<'_>,
) -> (RenderDevice, RenderQueue, RenderAdapterInfo) {
    let adapter = instance
        .request_adapter(request_adapter_options)
        .await
        .expect("Unable to find a GPU! Make sure you have installed required drivers!");

    let adapter_info = adapter.get_info();
    log::info!("{:?}", adapter_info);

    let trace_path = None;

    // Maybe get features and limits based on what is supported by the adapter/backend
    let features = options.features;
    let limits = options.limits.clone();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: options.device_label.as_ref().map(|a| a.as_ref()),
                features,
                limits,
            },
            trace_path,
        )
        .await
        .unwrap();
    let device = Arc::new(device);
    let queue = Arc::new(queue);
    (
        RenderDevice::from(device),
        RenderQueue(queue),
        RenderAdapterInfo(adapter_info),
    )
}

/// The context with all information required to interact with the GPU.
///
/// The [`RenderDevice`] is used to create render resources and the
/// the [`CommandEncoder`] is used to record a series of GPU operations.
pub struct RenderContext {
    pub render_device: RenderDevice,
    pub command_encoder: CommandEncoder,
}
