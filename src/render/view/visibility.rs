mod render_layers;

use cgm::SquareMatrix;
pub use render_layers::*;

use crate::{
    app::{App, Stage},
    asset::{Assets, Handle},
    ecs::{Commands, Component, Entity, Query, QuerySet, QueryState, Res, With, Without},
    render::{
        cameras::{Camera, CameraProjection, OrthographicProjection, PerspectiveProjection},
        mesh::Mesh,
        primitives::{Aabb, Frustum},
    },
    transform::GlobalTransform,
};

/// User indication of whether an entity is visible
#[derive(Component, Clone, Debug)]
pub struct Visibility {
    pub is_visible: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self { is_visible: true }
    }
}

/// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
#[derive(Component, Clone, Debug)]
pub struct ComputedVisibility {
    pub is_visible: bool,
}

impl Default for ComputedVisibility {
    fn default() -> Self {
        Self { is_visible: true }
    }
}

/// Use this component to opt-out of built-in frustum culling for Mesh entities
#[derive(Component)]
pub struct NoFrustumCulling;

#[derive(Clone, Component, Default, Debug)]
pub struct VisibleEntities {
    pub entities: Vec<Entity>,
}

impl VisibleEntities {
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Entity> {
        self.entities.iter()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

// TODO Make sure the systems are run in correct order (see Bevy) - must run after transform_propagate_system
pub fn visibility_plugin(app: &mut App) {
    app.add_system_to_stage(Stage::PostUpdate, &calculate_bounds)
        .add_system_to_stage(Stage::PostUpdate, &update_frusta::<OrthographicProjection>)
        .add_system_to_stage(Stage::PostUpdate, &update_frusta::<PerspectiveProjection>)
        .add_system_to_stage(Stage::PostUpdate, &check_visibility);
}

pub fn calculate_bounds(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    without_aabb: Query<(Entity, &Handle<Mesh>), (Without<Aabb>, Without<NoFrustumCulling>)>,
) {
    for (entity, mesh_handle) in without_aabb.iter() {
        if let Some(mesh) = meshes.get(mesh_handle) {
            if let Some(aabb) = mesh.compute_aabb() {
                commands.entity(entity).insert(aabb);
            }
        }
    }
}

pub fn update_frusta<T: Component + CameraProjection + Send + Sync + 'static>(
    mut views: Query<(&GlobalTransform, &T, &mut Frustum)>,
) {
    for (transform, projection, mut frustum) in views.iter_mut() {
        let view_projection =
            projection.get_projection_matrix() * transform.compute_matrix().inverse().unwrap();
        *frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            projection.far(),
        );
    }
}

pub fn check_visibility(
    mut view_query: Query<(&mut VisibleEntities, &Frustum, Option<&RenderLayers>), With<Camera>>,
    mut visible_entity_query: QuerySet<(
        QueryState<&mut ComputedVisibility>,
        QueryState<(
            Entity,
            &Visibility,
            &mut ComputedVisibility,
            Option<&RenderLayers>,
            Option<&Aabb>,
            Option<&NoFrustumCulling>,
            Option<&GlobalTransform>,
        )>,
    )>,
) {
    // Reset the computed visibility to false
    for mut computed_visibility in visible_entity_query.q0().iter_mut() {
        computed_visibility.is_visible = false;
    }

    for (mut visible_entities, frustum, maybe_view_mask) in view_query.iter_mut() {
        visible_entities.entities.clear();
        let view_mask = maybe_view_mask.copied().unwrap_or_default();

        for (
            entity,
            visibility,
            mut computed_visibility,
            maybe_entity_mask,
            maybe_aabb,
            maybe_no_frustum_culling,
            maybe_transform,
        ) in visible_entity_query.q1().iter_mut()
        {
            if !visibility.is_visible {
                continue;
            }

            let entity_mask = maybe_entity_mask.copied().unwrap_or_default();
            if !view_mask.intersects(&entity_mask) {
                continue;
            }

            // If we have an aabb and transform, do frustum culling
            if let (Some(aabb), None, Some(transform)) =
                (maybe_aabb, maybe_no_frustum_culling, maybe_transform)
            {
                if !frustum.intersects_obb(aabb, &transform.compute_matrix()) {
                    continue;
                }
            }

            computed_visibility.is_visible = true;
            visible_entities.entities.push(entity);
        }

        // TODO: check for big changes in visible entities len() vs capacity() (ex: 2x) and resize
        // to prevent holding unneeded memory
    }
}
