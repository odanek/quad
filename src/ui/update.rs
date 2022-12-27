use crate::{
    ecs::{Commands, Entity, Query, With, Without},
    sprite::Rect,
    transform::{Children, GlobalTransform, Parent, Transform},
};

use super::{CalculatedClip, Node, Overflow, Style};

/// The resolution of Z values for UI
pub const UI_Z_STEP: f32 = 0.001;

/// Updates transforms of nodes to fit with the z system
pub fn ui_z_system(
    root_node_query: Query<Entity, (With<Node>, Without<Parent>)>,
    mut node_query: Query<&mut Transform, With<Node>>,
    children_query: Query<&Children>,
) {
    let mut current_global_z = 0.0;
    for entity in root_node_query.iter() {
        current_global_z = update_hierarchy(
            &children_query,
            &mut node_query,
            entity,
            current_global_z,
            current_global_z,
        );
    }
}

fn update_hierarchy(
    children_query: &Query<&Children>,
    node_query: &mut Query<&mut Transform, With<Node>>,
    entity: Entity,
    parent_global_z: f32,
    mut current_global_z: f32,
) -> f32 {
    current_global_z += UI_Z_STEP;
    if let Ok(mut transform) = node_query.get_mut(entity) {
        let new_z = current_global_z - parent_global_z;
        // only trigger change detection when the new value is different
        if transform.translation.z != new_z {
            transform.translation.z = new_z;
        }
    }
    if let Ok(children) = children_query.get(entity) {
        let current_parent_global_z = current_global_z;
        for child in children.iter().cloned() {
            current_global_z = update_hierarchy(
                children_query,
                node_query,
                child,
                current_parent_global_z,
                current_global_z,
            );
        }
    }
    current_global_z
}

/// Updates clipping for all nodes
pub fn update_clipping_system(
    mut commands: Commands,
    root_node_query: Query<Entity, (With<Node>, Without<Parent>)>,
    mut node_query: Query<(&Node, &GlobalTransform, &Style, Option<&mut CalculatedClip>)>,
    children_query: Query<&Children>,
) {
    for root_node in &root_node_query {
        update_clipping(
            &mut commands,
            &children_query,
            &mut node_query,
            root_node,
            None,
        );
    }
}

fn update_clipping(
    commands: &mut Commands,
    children_query: &Query<&Children>,
    node_query: &mut Query<(&Node, &GlobalTransform, &Style, Option<&mut CalculatedClip>)>,
    entity: Entity,
    clip: Option<Rect>,
) {
    let (node, global_transform, style, calculated_clip) = node_query.get_mut(entity).unwrap();
    // Update this node's CalculatedClip component
    match (clip, calculated_clip) {
        (None, None) => {}
        (None, Some(_)) => {
            commands.entity(entity).remove::<CalculatedClip>();
        }
        (Some(clip), None) => {
            commands.entity(entity).insert(CalculatedClip { clip });
        }
        (Some(clip), Some(mut old_clip)) => {
            if old_clip.clip != clip {
                *old_clip = CalculatedClip { clip };
            }
        }
    }

    // Calculate new clip for its children
    let children_clip = match style.overflow {
        Overflow::Visible => clip,
        Overflow::Hidden => {
            let node_center = global_transform.translation.truncate();
            let node_rect = Rect::from_center_size(node_center, node.calculated_size);
            Some(clip.map_or(node_rect, |c| c.intersect(node_rect)))
        }
    };

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter().cloned() {
            update_clipping(commands, children_query, node_query, child, children_clip);
        }
    }
}
