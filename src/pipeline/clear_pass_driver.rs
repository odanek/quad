use crate::{
    ecs::World,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext},
        renderer::RenderContext,
    },
};

use super::clear_graph;

pub struct ClearPassDriverNode;

impl Node for ClearPassDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        graph.run_sub_graph(clear_graph::NAME, vec![])?;

        Ok(())
    }
}
