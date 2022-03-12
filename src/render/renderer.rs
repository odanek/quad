mod graph_runner;
mod render_device;

pub use graph_runner::*;
pub use render_device::*;

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use wgpu::{CommandEncoder, Instance, Queue, RequestAdapterOptions};

use crate::ecs::{ResMut, Resource, World};

use super::options::WgpuOptions;

/// Updates the [`RenderGraph`] with all of its nodes and then runs it to render the entire frame.
// pub fn render_system(world: &mut World) {
//     world.resource_scope(|world, mut graph: ResMut<RenderGraph>| {
//         graph.update(world);
//     });
//     let graph = world.resource::<RenderGraph>();
//     let render_device = world.resource::<RenderDevice>();
//     let render_queue = world.resource::<RenderQueue>();
//     RenderGraphRunner::run(
//         graph,
//         render_device.clone(), // TODO: is this clone really necessary?
//         render_queue,
//         world,
//     )
//     .unwrap();

//         // Remove ViewTarget components to ensure swap chain TextureViews are dropped.
//         // If all TextureViews aren't dropped before present, acquiring the next swap chain texture will fail.
//         let view_entities = world
//             .query_filtered::<Entity, With<ViewTarget>>()
//             .iter(world)
//             .collect::<Vec<_>>();
//         for view_entity in view_entities {
//             world.entity_mut(view_entity).remove::<ViewTarget>();
//         }

//         let mut windows = world.get_resource_mut::<ExtractedWindows>().unwrap();
//         for window in windows.values_mut() {
//             if let Some(texture_view) = window.swap_chain_texture.take() {
//                 if let Some(surface_texture) = texture_view.take_surface_texture() {
//                     surface_texture.present();
//                 }
//             }
//         }
// }

#[derive(Resource)]
pub struct RenderQueue(Arc<Queue>);

impl Deref for RenderQueue {
    type Target = Arc<Queue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct RenderInstance(Instance);

impl Deref for RenderInstance {
    type Target = Instance;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// `wgpu::Features` that are not automatically enabled due to having possibly-negative side effects.
/// `MAPPABLE_PRIMARY_BUFFERS` can have a significant, negative performance impact so should not be
/// automatically enabled.
pub const DEFAULT_DISABLED_WGPU_FEATURES: wgpu::Features = wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;

pub async fn initialize_renderer(
    instance: &Instance,
    options: &mut WgpuOptions,
    request_adapter_options: &RequestAdapterOptions<'_>,
) -> (RenderDevice, RenderQueue) {
    let adapter = instance
        .request_adapter(request_adapter_options)
        .await
        .expect("Unable to find a GPU! Make sure you have installed required drivers!");

    log::info!("{:?}", adapter.get_info());

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: options.device_label.as_ref().map(|a| a.as_ref()),
                features: options.features,
                limits: options.limits.clone(),
            },
            None,
        )
        .await
        .unwrap();
    let device = Arc::new(device);
    let queue = Arc::new(queue);
    (RenderDevice::from(device), RenderQueue(queue))
}

pub struct RenderContext {
    pub render_device: RenderDevice,
    pub command_encoder: CommandEncoder,
}
