use wgpu::{LoadOp, Operations, RenderPassDescriptor, StoreOp};

use crate::{
    ecs::{QueryState, With, World},
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::{DrawFunctions, RenderPhase, TrackedRenderPass},
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget},
    },
};

use super::Transparent2d;

pub struct MainPass2dNode {
    query:
        QueryState<(&'static RenderPhase<Transparent2d>, &'static ViewTarget), With<ExtractedView>>,
}

impl MainPass2dNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for MainPass2dNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(MainPass2dNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (transparent_phase, target) = self
            .query
            .get_manual(world, view_entity)
            .expect("view entity should exist");

        let color_attachment = target.get_color_attachment(Operations {
            load: LoadOp::Load,
            store: StoreOp::Store,
        });
        let pass_descriptor = RenderPassDescriptor {
            label: Some("main_pass_2d"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        let draw_functions = world.resource::<DrawFunctions<Transparent2d>>();

        let render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);

        let mut draw_functions = draw_functions.write();
        let mut tracked_pass = TrackedRenderPass::new(render_pass);
        for item in &transparent_phase.items {
            let draw_function = draw_functions.get_mut(item.draw_function).unwrap();
            draw_function.draw(world, &mut tracked_pass, view_entity, item);
        }
        Ok(())
    }
}
