use crate::ecs::{Entity, Query, With, Without};

use super::{Children, GlobalTransform, Parent, Transform};

pub fn transform_propagate_system(
    mut root_query: Query<
        (Option<&Children>, &Transform, &mut GlobalTransform),
        Without<Parent>,
    >,
    mut transform_query: Query<(&Transform, &mut GlobalTransform), With<Parent>>,    
    children_query: Query<Option<&Children>, (With<Parent>, With<GlobalTransform>)>,
) {
    for (children, transform, global_transform) in root_query.iter_mut() {
        *global_transform = GlobalTransform::from(*transform);

        if let Some(children) = children {
            for child in children.0.iter() {
                propagate_recursive(
                    &global_transform,
                    &mut transform_query,
                    &children_query,
                    *child,
                );
            }
        }
    }
}

fn propagate_recursive(
    parent: &GlobalTransform,
    transform_query: &mut Query<(&Transform, &mut GlobalTransform), With<Parent>>,
    children_query: &Query<Option<&Children>, (With<Parent>, With<GlobalTransform>)>,
    entity: Entity,
) {
    let global_matrix = {
        if let Ok((transform, global_transform)) = transform_query.get_mut(entity) {
            *global_transform = parent.mul_transform(*transform);
            *global_transform
        } else {
            return;
        }
    };

    if let Ok(Some(children)) = children_query.get(entity) {
        for child in children.0.iter() {
            propagate_recursive(
                &global_matrix,
                transform_query,
                children_query,
                *child,
            );
        }
    }
}