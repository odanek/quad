use cgm::SquareMatrix;

use crate::{
    app::{App, MainStage},
    ecs::{Bundle, Component, Entity, Query, With, Without},
    render::{
        cameras::{Camera, CameraProjection, OrthographicProjection},
        primitives::Frustum,
    },
    transform::{Children, GlobalTransform, Parent},
};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Inherited,
    Hidden,
    Visible,
}

impl std::cmp::PartialEq<Visibility> for &Visibility {
    #[inline]
    fn eq(&self, other: &Visibility) -> bool {
        **self == *other
    }
}
impl std::cmp::PartialEq<&Visibility> for Visibility {
    #[inline]
    fn eq(&self, other: &&Visibility) -> bool {
        *self == **other
    }
}

bitflags::bitflags! {
    pub(super) struct ComputedVisibilityFlags: u8 {
        const VISIBLE_IN_VIEW = 1 << 0;
        const VISIBLE_IN_HIERARCHY = 1 << 1;
    }
}

/// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct ComputedVisibility {
    flags: ComputedVisibilityFlags,
}

impl Default for ComputedVisibility {
    fn default() -> Self {
        Self::HIDDEN
    }
}

impl ComputedVisibility {
    /// A [`ComputedVisibility`], set as invisible.
    pub const HIDDEN: Self = ComputedVisibility {
        flags: ComputedVisibilityFlags::empty(),
    };

    /// Whether this entity is visible to something this frame. This is true if and only if [`Self::is_visible_in_hierarchy`] and [`Self::is_visible_in_view`]
    /// are true. This is the canonical method to call to determine if an entity should be drawn.
    /// This value is updated in [`CoreStage::PostUpdate`] during the [`VisibilitySystems::CheckVisibility`] system label. Reading it from the
    /// [`CoreStage::Update`] stage will yield the value from the previous frame.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.flags.bits == ComputedVisibilityFlags::all().bits
    }

    /// Whether this entity is visible in the entity hierarchy, which is determined by the [`Visibility`] component.
    /// This takes into account "visibility inheritance". If any of this entity's ancestors (see [`Parent`]) are hidden, this entity
    /// will be hidden as well. This value is updated in the [`CoreStage::PostUpdate`] stage in the
    /// [`VisibilitySystems::VisibilityPropagate`] system label.
    #[inline]
    pub fn is_visible_in_hierarchy(&self) -> bool {
        self.flags
            .contains(ComputedVisibilityFlags::VISIBLE_IN_HIERARCHY)
    }

    /// Whether this entity is visible in _any_ view (Cameras, Lights, etc). Each entity type (and view type) should choose how to set this
    /// value. For cameras and drawn entities, this will take into account [`RenderLayers`].
    ///
    /// This value is reset to `false` every frame in [`VisibilitySystems::VisibilityPropagate`] during [`CoreStage::PostUpdate`].
    /// Each entity type then chooses how to set this field in the [`CoreStage::PostUpdate`] stage in the
    /// [`VisibilitySystems::CheckVisibility`] system label. Meshes might use frustum culling to decide if they are visible in a view.
    /// Other entities might just set this to `true` every frame.
    #[inline]
    pub fn is_visible_in_view(&self) -> bool {
        self.flags
            .contains(ComputedVisibilityFlags::VISIBLE_IN_VIEW)
    }

    /// Sets `is_visible_in_view` to `true`. This is not reversible for a given frame, as it encodes whether or not this is visible in
    /// _any_ view. This will be automatically reset to `false` every frame in [`VisibilitySystems::VisibilityPropagate`] and then set
    /// to the proper value in [`VisibilitySystems::CheckVisibility`]. This should _only_ be set in systems with the [`VisibilitySystems::CheckVisibility`]
    /// label. Don't call this unless you are defining a custom visibility system. For normal user-defined entity visibility, see [`Visibility`].
    #[inline]
    pub fn set_visible_in_view(&mut self) {
        self.flags.insert(ComputedVisibilityFlags::VISIBLE_IN_VIEW);
    }

    #[inline]
    fn reset(&mut self, visible_in_hierarchy: bool) {
        self.flags = if visible_in_hierarchy {
            ComputedVisibilityFlags::VISIBLE_IN_HIERARCHY
        } else {
            ComputedVisibilityFlags::empty()
        };
    }
}

#[derive(Bundle, Debug, Default)]
pub struct VisibilityBundle {
    /// The visibility of the entity.
    pub visibility: Visibility,
    /// The computed visibility of the entity.
    pub computed: ComputedVisibility,
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
    .add_system_to_stage(MainStage::PostTransformUpdate, visibility_propagate_system)
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

#[allow(clippy::type_complexity)]
fn visibility_propagate_system(
    mut root_query: Query<
        (
            Option<&Children>,
            &Visibility,
            &mut ComputedVisibility,
            Entity,
        ),
        Without<Parent>,
    >,
    mut visibility_query: Query<(&Visibility, &mut ComputedVisibility, &Parent)>,
    children_query: Query<&Children, (With<Parent>, With<Visibility>, With<ComputedVisibility>)>,
) {
    for (children, visibility, mut computed_visibility, entity) in root_query.iter_mut() {
        // reset "view" visibility here ... if this entity should be drawn a future system should set this to true
        computed_visibility
            .reset(visibility == Visibility::Inherited || visibility == Visibility::Visible);
        if let Some(children) = children {
            for child in children.iter() {
                let _ = propagate_recursive(
                    computed_visibility.is_visible_in_hierarchy(),
                    &mut visibility_query,
                    &children_query,
                    *child,
                    entity,
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn propagate_recursive(
    parent_visible: bool,
    visibility_query: &mut Query<(&Visibility, &mut ComputedVisibility, &Parent)>,
    children_query: &Query<&Children, (With<Parent>, With<Visibility>, With<ComputedVisibility>)>,
    entity: Entity,
    expected_parent: Entity,
    // BLOCKED: https://github.com/rust-lang/rust/issues/31436
    // We use a result here to use the `?` operator. Ideally we'd use a try block instead
) -> Result<(), ()> {
    let is_visible = {
        let (visibility, mut computed_visibility, child_parent) =
            visibility_query.get_mut(entity).map_err(drop)?;
        assert_eq!(
            child_parent.get(), expected_parent,
            "Malformed hierarchy. This probably means that your hierarchy has been improperly maintained, or contains a cycle"
        );
        let visible_in_hierarchy = (parent_visible && visibility == Visibility::Inherited)
            || visibility == Visibility::Visible;
        // reset "view" visibility here ... if this entity should be drawn a future system should set this to true
        computed_visibility.reset(visible_in_hierarchy);
        visible_in_hierarchy
    };

    for child in children_query.get(entity).map_err(drop)?.iter() {
        let _ = propagate_recursive(is_visible, visibility_query, children_query, *child, entity);
    }
    Ok(())
}

// TODO Sprites don't have Aabb so they are not culled?
#[allow(clippy::type_complexity)]
pub fn check_visibility(
    mut view_query: Query<(&mut VisibleEntities, &Frustum), With<Camera>>,
    mut visible_no_aabb_query: Query<(Entity, &mut ComputedVisibility)>,
) {
    for (mut visible_entities, _frustum) in &mut view_query {
        visible_entities.entities.clear();

        for (entity, mut computed_visibility) in &mut visible_no_aabb_query {
            if !computed_visibility.is_visible_in_hierarchy() {
                continue;
            }

            computed_visibility.set_visible_in_view();
            visible_entities.entities.push(entity);
        }
    }
}
