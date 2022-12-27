use cgm::SquareMatrix;

use crate::{
    app::{App, MainStage},
    ecs::{Component, Entity, ParamSet, Query, With},
    render::{
        cameras::{Camera, CameraProjection, OrthographicProjection},
        primitives::{Frustum},
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

pub fn visibility_plugin(app: &mut App) {
    app.add_system_to_stage(
        MainStage::PostTransformUpdate,
        update_frusta::<OrthographicProjection>,
    ) // Must run after transform_propagate_system
    .add_system_to_stage(MainStage::PostTransformUpdate, check_visibility); // After calculate_bounds and update_frust
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

// TODO Sprites don't have Aabb so they are not culled?
#[allow(clippy::type_complexity)]
pub fn check_visibility(
    mut view_query: Query<(&mut VisibleEntities, &Frustum), With<Camera>>,
    mut visible_entity_query: ParamSet<(
        Query<&mut ComputedVisibility>,
        Query<(
            Entity,
            &Visibility,
            &mut ComputedVisibility,
            Option<&GlobalTransform>,
        )>,
    )>,
) {
    // Reset the computed visibility to false
    for mut computed_visibility in visible_entity_query.p0().iter_mut() {
        computed_visibility.is_visible = false;
    }

    for (mut visible_entities, _frustum) in view_query.iter_mut() {
        visible_entities.entities.clear();

        for (entity, visibility, mut computed_visibility, _maybe_transform) in
            visible_entity_query.p1().iter_mut()
        {
            if !visibility.is_visible {
                continue;
            }

            computed_visibility.is_visible = true;
            visible_entities.entities.push(entity);
        }

        // TODO: check for big changes in visible entities len() vs capacity() (ex: 2x) and resize
        // to prevent holding unneeded memory
    }
}
