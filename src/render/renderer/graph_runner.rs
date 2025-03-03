use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};
use thiserror::Error;

use crate::{
    ecs::World,
    render::render_graph::{
        Edge, NodeId, NodeRunError, NodeState, RenderGraph, RenderGraphContext, SlotLabel,
        SlotType, SlotValue,
    },
};

use super::{RenderContext, RenderDevice};

pub(crate) struct RenderGraphRunner;

#[derive(Error, Debug)]
pub enum RenderGraphRunnerError {
    #[error(transparent)]
    NodeRunError(#[from] NodeRunError),
    #[error("node output slot not set (index {slot_index}, name {slot_name})")]
    EmptyNodeOutputSlot {
        type_name: &'static str,
        slot_index: usize,
        slot_name: Cow<'static, str>,
    },
    #[error(
        "graph (name: '{graph_name:?}') could not be run because slot '{slot_name}' at index {slot_index} has no value"
    )]
    MissingInput {
        slot_index: usize,
        slot_name: Cow<'static, str>,
        graph_name: Option<Cow<'static, str>>,
    },
    #[error("attempted to use the wrong type for input slot")]
    MismatchedInputSlotType {
        slot_index: usize,
        label: SlotLabel,
        expected: SlotType,
        actual: SlotType,
    },
}

impl RenderGraphRunner {
    pub fn run(
        graph: &RenderGraph,
        render_device: RenderDevice,
        queue: &wgpu::Queue,
        world: &World,
    ) -> Result<(), RenderGraphRunnerError> {
        let command_encoder =
            render_device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut render_context = RenderContext {
            render_device,
            command_encoder,
        };

        Self::run_graph(graph, None, &mut render_context, world, &[])?;
        {
            queue.submit(vec![render_context.command_encoder.finish()]);
        }
        Ok(())
    }

    fn run_graph(
        graph: &RenderGraph,
        graph_name: Option<Cow<'static, str>>,
        render_context: &mut RenderContext,
        world: &World,
        inputs: &[SlotValue],
    ) -> Result<(), RenderGraphRunnerError> {
        let mut node_outputs: HashMap<NodeId, Vec<SlotValue>> = HashMap::default();

        // Queue up nodes without inputs, which can be run immediately
        let mut node_queue: VecDeque<&NodeState> = graph
            .iter_nodes()
            .filter(|node| node.input_slots.is_empty())
            .collect();

        // pass inputs into the graph
        if let Some(input_node) = graph.input_node() {
            let mut input_values: Vec<SlotValue> = Vec::with_capacity(4);
            for (i, input_slot) in input_node.input_slots.iter().enumerate() {
                if let Some(input_value) = inputs.get(i) {
                    if input_slot.slot_type != input_value.slot_type() {
                        return Err(RenderGraphRunnerError::MismatchedInputSlotType {
                            slot_index: i,
                            actual: input_value.slot_type(),
                            expected: input_slot.slot_type,
                            label: input_slot.name.clone().into(),
                        });
                    } else {
                        input_values.push(input_value.clone());
                    }
                } else {
                    return Err(RenderGraphRunnerError::MissingInput {
                        slot_index: i,
                        slot_name: input_slot.name.clone(),
                        graph_name: graph_name.clone(),
                    });
                }
            }

            node_outputs.insert(input_node.id, input_values);

            for (_, node_state) in graph.iter_node_outputs(input_node.id).expect("node exists") {
                node_queue.push_front(node_state);
            }
        }

        'handle_node: while let Some(node_state) = node_queue.pop_back() {
            // skip nodes that are already processed
            if node_outputs.contains_key(&node_state.id) {
                continue;
            }

            let mut slot_indices_and_inputs: Vec<(usize, SlotValue)> = Vec::with_capacity(4);
            // check if all dependencies have finished running
            for (edge, input_node) in graph
                .iter_node_inputs(node_state.id)
                .expect("node is in graph")
            {
                match edge {
                    Edge::SlotEdge {
                        output_index,
                        input_index,
                        ..
                    } => {
                        if let Some(outputs) = node_outputs.get(&input_node.id) {
                            slot_indices_and_inputs
                                .push((*input_index, outputs[*output_index].clone()));
                        } else {
                            node_queue.push_front(node_state);
                            continue 'handle_node;
                        }
                    }
                    Edge::NodeEdge { .. } => {
                        if !node_outputs.contains_key(&input_node.id) {
                            node_queue.push_front(node_state);
                            continue 'handle_node;
                        }
                    }
                }
            }

            // construct final sorted input list
            slot_indices_and_inputs.sort_by_key(|(index, _)| *index);
            let inputs: Vec<SlotValue> = slot_indices_and_inputs
                .into_iter()
                .map(|(_, value)| value)
                .collect();

            assert_eq!(inputs.len(), node_state.input_slots.len());

            let mut outputs: Vec<Option<SlotValue>> = vec![None; node_state.output_slots.len()];
            {
                let mut context = RenderGraphContext::new(graph, node_state, &inputs, &mut outputs);

                node_state.node.run(&mut context, render_context, world)?;

                for run_sub_graph in context.finish() {
                    let sub_graph = graph
                        .get_sub_graph(&run_sub_graph.name)
                        .expect("sub graph exists because it was validated when queued.");
                    Self::run_graph(
                        sub_graph,
                        Some(run_sub_graph.name),
                        render_context,
                        world,
                        &run_sub_graph.inputs,
                    )?;
                }
            }

            let mut values: Vec<SlotValue> = Vec::with_capacity(4);
            for (i, output) in outputs.into_iter().enumerate() {
                if let Some(value) = output {
                    values.push(value);
                } else {
                    let empty_slot = node_state.output_slots.get_slot(i).unwrap();
                    return Err(RenderGraphRunnerError::EmptyNodeOutputSlot {
                        type_name: node_state.type_name,
                        slot_index: i,
                        slot_name: empty_slot.name.clone(),
                    });
                }
            }
            node_outputs.insert(node_state.id, values);

            for (_, node_state) in graph.iter_node_outputs(node_state.id).expect("node exists") {
                node_queue.push_front(node_state);
            }
        }

        Ok(())
    }
}
