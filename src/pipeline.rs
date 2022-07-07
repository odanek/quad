mod clear_pass;
mod clear_pass_driver;
mod main_pass_2d;
mod main_pass_driver;

pub use clear_pass::*;
pub use clear_pass_driver::*;
pub use main_pass_2d::*;
pub use main_pass_driver::*;

use std::{collections::HashMap, ops::Range};

use crate::{
    app::{App, RenderStage},
    ecs::{Commands, Entity, IntoSystem, Res, ResMut, Resource},
    render::{
        cameras::{ActiveCamera, Camera2d, RenderTarget},
        color::Color,
        render_graph::{EmptyNode, RenderGraph, SlotInfo, SlotType},
        render_phase::{
            batch_phase_system, sort_phase_system, BatchedPhaseItem, CachedPipelinePhaseItem,
            DrawFunctionId, DrawFunctions, EntityPhaseItem, PhaseItem, RenderPhase,
        },
        render_resource::CachedPipelineId,
        RenderWorld,
    },
    ty::FloatOrd,
};

pub mod prelude {
    pub use crate::pipeline::ClearColor;
}

/// When used as a resource, sets the color that is used to clear the screen between frames.
///
/// This color appears as the "background" color for simple apps, when
/// there are portions of the screen with nothing rendered.
#[derive(Clone, Debug, Resource)]
pub struct ClearColor(pub Color);

impl Default for ClearColor {
    fn default() -> Self {
        Self(Color::rgb(0.4, 0.4, 0.4))
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub struct RenderTargetClearColors {
    colors: HashMap<RenderTarget, Color>,
}

impl RenderTargetClearColors {
    pub fn get(&self, target: &RenderTarget) -> Option<&Color> {
        self.colors.get(target)
    }
    pub fn insert(&mut self, target: RenderTarget, color: Color) {
        self.colors.insert(target, color);
    }
}

// Plugins that contribute to the RenderGraph should use the following label conventions:
// 1. Graph modules should have a NAME, input module, and node module (where relevant)
// 2. The "top level" graph is the plugin module root. Just add things like `pub mod node` directly under the plugin module
// 3. "sub graph" modules should be nested beneath their parent graph module

pub mod node {
    pub const MAIN_PASS_DEPENDENCIES: &str = "main_pass_dependencies";
    pub const MAIN_PASS_DRIVER: &str = "main_pass_driver";
    pub const CLEAR_PASS_DRIVER: &str = "clear_pass_driver";
}

pub mod draw_2d_graph {
    pub const NAME: &str = "draw_2d";
    pub mod input {
        pub const VIEW_ENTITY: &str = "view_entity";
    }
    pub mod node {
        pub const MAIN_PASS: &str = "main_pass";
    }
}

pub mod clear_graph {
    pub const NAME: &str = "clear";
    pub mod node {
        pub const CLEAR_PASS: &str = "clear_pass";
    }
}

pub fn core_pipeline_plugin(app: &mut App, render_app: &mut App) {
    app.init_resource::<ClearColor>()
        .init_resource::<RenderTargetClearColors>();

    render_app
        .init_resource::<DrawFunctions<Transparent2d>>()
        .add_system_to_stage(
            RenderStage::Extract,
            extract_clear_color.system(&mut app.world),
        )
        .add_system_to_stage(
            RenderStage::Extract,
            extract_core_pipeline_camera_phases.system(&mut app.world),
        )
        .add_system_to_stage(RenderStage::PhaseSort, &sort_phase_system::<Transparent2d>)
        .add_system_to_stage(RenderStage::PhaseSort, &batch_phase_system::<Transparent2d>);

    let clear_pass_node = ClearPassNode::new(&mut render_app.world);
    let pass_node_2d = MainPass2dNode::new(&mut render_app.world);

    let mut graph = render_app.world.resource_mut::<RenderGraph>();

    let mut draw_2d_graph = RenderGraph::default();
    draw_2d_graph.add_node(draw_2d_graph::node::MAIN_PASS, pass_node_2d);
    let input_node_id = draw_2d_graph.set_input(vec![SlotInfo::new(
        draw_2d_graph::input::VIEW_ENTITY,
        SlotType::Entity,
    )]);
    draw_2d_graph
        .add_slot_edge(
            input_node_id,
            draw_2d_graph::input::VIEW_ENTITY,
            draw_2d_graph::node::MAIN_PASS,
            MainPass2dNode::IN_VIEW,
        )
        .unwrap();
    graph.add_sub_graph(draw_2d_graph::NAME, draw_2d_graph);

    let mut clear_graph = RenderGraph::default();
    clear_graph.add_node(clear_graph::node::CLEAR_PASS, clear_pass_node);
    graph.add_sub_graph(clear_graph::NAME, clear_graph);

    graph.add_node(node::MAIN_PASS_DEPENDENCIES, EmptyNode);
    graph.add_node(node::MAIN_PASS_DRIVER, MainPassDriverNode);
    graph
        .add_node_edge(node::MAIN_PASS_DEPENDENCIES, node::MAIN_PASS_DRIVER)
        .unwrap();
    graph.add_node(node::CLEAR_PASS_DRIVER, ClearPassDriverNode);
    graph
        .add_node_edge(node::CLEAR_PASS_DRIVER, node::MAIN_PASS_DRIVER)
        .unwrap();
}

pub struct Transparent2d {
    pub sort_key: FloatOrd,
    pub entity: Entity,
    pub pipeline: CachedPipelineId,
    pub draw_function: DrawFunctionId,
    /// Range in the vertex buffer of this item
    pub batch_range: Option<Range<u32>>,
}

impl PhaseItem for Transparent2d {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
}

impl EntityPhaseItem for Transparent2d {
    #[inline]
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl CachedPipelinePhaseItem for Transparent2d {
    #[inline]
    fn cached_pipeline(&self) -> CachedPipelineId {
        self.pipeline
    }
}

impl BatchedPhaseItem for Transparent2d {
    fn batch_range(&self) -> &Option<Range<u32>> {
        &self.batch_range
    }

    fn batch_range_mut(&mut self) -> &mut Option<Range<u32>> {
        &mut self.batch_range
    }
}

pub fn extract_clear_color(
    clear_color: Res<ClearColor>,
    clear_colors: Res<RenderTargetClearColors>,
    mut render_world: ResMut<RenderWorld>,
) {
    // If the clear color has changed
    if clear_color.is_changed() {
        // Update the clear color resource in the render world
        render_world.insert_resource(clear_color.clone());
    }

    // If the clear color has changed
    if clear_colors.is_changed() {
        // Update the clear color resource in the render world
        render_world.insert_resource(clear_colors.clone());
    }
}

pub fn extract_core_pipeline_camera_phases(
    mut commands: Commands,
    active_2d: Res<ActiveCamera<Camera2d>>,
) {
    if let Some(entity) = active_2d.get() {
        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<Transparent2d>::default());
    }
}
