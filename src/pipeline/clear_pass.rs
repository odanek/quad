use std::collections::HashSet;

use wgpu::{
    LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp,
};

use crate::{
    ecs::{QueryState, With, World},
    render::{
        cameras::{ExtractedCamera, RenderTarget},
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo},
        renderer::RenderContext,
        texture::Image,
        view::{window::ExtractedWindows, ExtractedView, ViewDepthTexture, ViewTarget},
    },
};

use super::{ClearColor, RenderTargetClearColors};

pub struct ClearPassNode {
    #[allow(clippy::type_complexity)]
    query: QueryState<
        (
            &'static ViewTarget,
            Option<&'static ViewDepthTexture>,
            Option<&'static ExtractedCamera>,
        ),
        With<ExtractedView>,
    >,
}

impl ClearPassNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for ClearPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let mut cleared_targets = HashSet::new();
        let clear_color = world.resource::<ClearColor>();
        let render_target_clear_colors = world.resource::<RenderTargetClearColors>();

        // This gets all ViewTargets and ViewDepthTextures and clears its attachments
        // TODO: This has the potential to clear the same target multiple times, if there
        // are multiple views drawing to the same target. This should be fixed when we make
        // clearing happen on "render targets" instead of "views" (see the TODO below for more context).
        for (target, depth, camera) in self.query.iter_manual(world) {
            let mut color = &clear_color.0;
            if let Some(camera) = camera {
                cleared_targets.insert(&camera.target);
                if let Some(target_color) = render_target_clear_colors.get(&camera.target) {
                    color = target_color;
                }
            }
            let color_attachment = target.get_color_attachment(Operations {
                load: LoadOp::Clear((*color).into()),
                store: StoreOp::Store,
            });
            let pass_descriptor = RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: depth.map(|depth| RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(0.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            };

            render_context
                .command_encoder
                .begin_render_pass(&pass_descriptor);
        }

        // TODO: This is a hack to ensure we don't call present() on frames without any work,
        // which will cause panics. The real fix here is to clear "render targets" directly
        // instead of "views". This should be removed once full RenderTargets are implemented.
        let windows = world.resource::<ExtractedWindows>();
        let images = world.resource::<RenderAssets<Image>>();
        for target in render_target_clear_colors.colors.keys().cloned().chain(
            windows
                .values()
                .map(|window| RenderTarget::Window(window.id)),
        ) {
            // skip windows that have already been cleared
            if cleared_targets.contains(&target) {
                continue;
            }
            let color_attachment = RenderPassColorAttachment {
                view: target
                    .get_texture_view(windows.as_ref(), images.as_ref())
                    .unwrap(),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(
                        (*render_target_clear_colors
                            .get(&target)
                            .unwrap_or(&clear_color.0))
                        .into(),
                    ),
                    store: StoreOp::Store,
                },
            };
            let pass_descriptor = RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };

            render_context
                .command_encoder
                .begin_render_pass(&pass_descriptor);
        }

        Ok(())
    }
}
