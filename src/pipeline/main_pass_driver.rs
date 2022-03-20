use crate::{
    ecs::World,
    render::{
        cameras::{ActiveCamera, Camera2d},
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotValue},
        renderer::RenderContext,
    },
};

use super::draw_2d_graph;

pub struct MainPassDriverNode;

impl Node for MainPassDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(camera_2d) = world.resource::<ActiveCamera<Camera2d>>().get() {
            graph.run_sub_graph(draw_2d_graph::NAME, vec![SlotValue::Entity(camera_2d)])?;
        }

        Ok(())
    }
}
