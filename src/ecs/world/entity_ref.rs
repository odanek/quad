use std::any::TypeId;

use crate::{
    ecs::{
        component::{Bundle, BundleInfo, CmptMut, Component, ComponentStatus, Components},
        entity::{Archetype, ArchetypeId, Archetypes, Entities, EntityLocation},
        storage::Storages,
        system::SystemTicks,
        Entity,
    },
    transform::{Children, Parent},
};

use super::World;

pub struct EntityRef<'w> {
    world: &'w World,
    entity: Entity,
    location: EntityLocation,
}

impl<'w> EntityRef<'w> {
    #[inline]
    pub(crate) fn new(world: &'w World, entity: Entity, location: EntityLocation) -> Self {
        EntityRef {
            world,
            entity,
            location,
        }
    }

    #[inline]
    pub fn id(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub(crate) fn archetype(&self) -> &Archetype {
        &self.world.archetypes[self.location.archetype_id]
    }

    #[inline]
    pub fn contains<T: Component>(&self) -> bool {
        self.world
            .components()
            .get_id(TypeId::of::<T>())
            .map_or(false, |id| self.archetype().contains(id))
    }

    #[inline]
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.world.get_component(self.location)
    }

    #[inline]
    pub fn parent(&self) -> Option<Entity> {
        self.get::<Parent>().map(|parent| parent.0)
    }

    #[inline]
    pub fn children(&self) -> Option<&[Entity]> {
        self.get::<Children>().map(|children| children.0.as_slice())
    }
}

pub struct EntityMut<'w> {
    world: &'w mut World,
    entity: Entity,
    location: EntityLocation,
}

impl<'w> EntityMut<'w> {
    #[inline]
    pub(crate) fn new(world: &'w mut World, entity: Entity, location: EntityLocation) -> Self {
        EntityMut {
            world,
            entity,
            location,
        }
    }

    #[inline]
    pub fn id(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub(crate) fn archetype(&self) -> &Archetype {
        &self.world.archetypes[self.location.archetype_id]
    }

    #[inline]
    pub fn contains<T: Component>(&self) -> bool {
        self.world
            .components()
            .get_id(TypeId::of::<T>())
            .map_or(false, |id| self.archetype().contains(id))
    }

    #[inline]
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.world.get_component(self.location)
    }

    #[inline]
    pub fn get_mut<T: Component>(&mut self) -> Option<CmptMut<T>> {
        unsafe {
            self.world
                .get_component_unchecked_mut::<T>(self.location)
                .map(|(data, ticks)| {
                    let last_change_tick = self.world.last_change_tick();
                    let change_tick = self.world.change_tick();
                    CmptMut::new(
                        &mut *data.cast::<T>(),
                        &mut *ticks,
                        SystemTicks::new(last_change_tick, change_tick),
                    )
                })
        }
    }

    // TODO: move relevant methods to World (add/remove bundle)
    pub fn insert_bundle<T: Bundle>(&mut self, bundle: T) -> &mut Self {
        let change_tick = self.world.change_tick();
        let entity = self.entity;
        let entities = &mut self.world.entities;
        let archetypes = &mut self.world.archetypes;
        let components = &mut self.world.components;
        let storages = &mut self.world.storages;

        let bundle_info = self.world.bundles.init_info::<T>(components);
        let current_location = self.location;

        let new_location = unsafe {
            get_insert_bundle_info(
                entities,
                archetypes,
                components,
                storages,
                bundle_info,
                current_location,
                entity,
            )
        };
        self.location = new_location;

        let edge = archetypes[current_location.archetype_id]
            .edges()
            .get_add_bundle(bundle_info.id)
            .unwrap();
        let archetype = &archetypes[new_location.archetype_id];
        let table = &mut storages.tables[archetype.table_id()];

        unsafe {
            bundle_info.write_components(
                table,
                new_location.index,
                bundle,
                &edge.bundle_status,
                change_tick,
            )
        };

        self
    }

    pub fn insert(&mut self, value: impl Component) -> &mut Self {
        self.insert_bundle((value,))
    }

    pub fn remove_bundle<T: Bundle>(&mut self) -> Option<T> {
        let archetypes = &mut self.world.archetypes;
        let storages = &mut self.world.storages;
        let components = &mut self.world.components;
        let entities = &mut self.world.entities;
        let removed_components = &mut self.world.removed_components;

        let entity = self.entity;
        let bundle_info = self.world.bundles.init_info::<T>(components);
        let old_location = self.location;
        let new_archetype_id = remove_bundle_from_archetype(
            archetypes,
            storages,
            components,
            old_location.archetype_id,
            bundle_info,
            false,
        )?;

        let old_archetype_id = old_location.archetype_id;
        let old_archetype = &mut archetypes[old_archetype_id];
        let old_table = &storages.tables[old_archetype.table_id()];
        let mut bundle_components = bundle_info.component_ids.iter().cloned();

        let result = unsafe {
            T::from_components(|| {
                let component_id = bundle_components.next().unwrap();

                removed_components
                    .entry(component_id)
                    .or_insert_with(Vec::new)
                    .push(entity);

                let table = old_table;
                let column = table
                    .get_column(component_id)
                    .expect("The entity does not contain given component");
                column.get_unchecked(old_location.index)
            })
        };

        if new_archetype_id == old_archetype_id {
            return Some(result);
        }

        self.location = move_entity_after_remove(
            false,
            self.entity,
            old_location,
            new_archetype_id,
            archetypes,
            storages,
            entities,
        );
        Some(result)
    }

    pub fn remove_bundle_intersection<T: Bundle>(&mut self) {
        let archetypes = &mut self.world.archetypes;
        let storages = &mut self.world.storages;
        let components = &mut self.world.components;
        let entities = &mut self.world.entities;
        let removed_components = &mut self.world.removed_components;

        let bundle_info = self.world.bundles.init_info::<T>(components);
        let old_location = self.location;
        let old_archetype_id = old_location.archetype_id;
        let new_archetype_id = remove_bundle_from_archetype(
            archetypes,
            storages,
            components,
            old_location.archetype_id,
            bundle_info,
            true,
        )
        .expect("intersections should always return a result");

        if new_archetype_id == old_archetype_id {
            return;
        }

        let entity = self.entity;
        let old_archetype = &mut archetypes[old_archetype_id];
        for component_id in bundle_info.component_ids.iter().cloned() {
            if old_archetype.contains(component_id) {
                removed_components
                    .entry(component_id)
                    .or_insert_with(Vec::new)
                    .push(entity);
            }
        }

        self.location = move_entity_after_remove(
            true,
            self.entity,
            old_location,
            new_archetype_id,
            archetypes,
            storages,
            entities,
        );
    }

    pub fn remove<T: Component>(&mut self) -> Option<T> {
        self.remove_bundle::<(T,)>().map(|v| v.0)
    }

    pub fn despawn(mut self) {
        self.remove_from_parent();
        if let Some(mut children) = self.get_mut::<Children>() {
            for child in std::mem::take(&mut children.0) {
                self.world.entity_mut(child).remove::<Parent>();
            }
        }
        despawn_self(self.world, self.entity);
    }

    pub fn despawn_recursive(mut self) {
        self.remove_from_parent();
        despawn_recursive(self.world, self.entity);
    }

    #[inline]
    pub fn parent(&self) -> Option<Entity> {
        self.get::<Parent>().map(|parent| parent.0)
    }

    #[inline]
    pub fn children(&self) -> Option<&[Entity]> {
        self.get::<Children>().map(|children| children.0.as_slice())
    }

    pub fn push_child(&mut self, child: Entity) -> &mut Self {
        // TODO: What if child already has a parent?
        // TODO: Uniqueness
        self.world.entity_mut(child).insert(Parent(self.entity));
        self.refresh_location();

        if let Some(mut children_component) = self.get_mut::<Children>() {
            children_component.0.push(child);
        } else {
            self.insert(Children::with(&[child]));
        }

        self
    }

    pub fn push_children(&mut self, children: &[Entity]) -> &mut Self {
        // TODO: What if child already has a parent?
        // TODO: Uniqueness
        let parent = self.entity;
        for child in children.iter() {
            self.world.entity_mut(*child).insert(Parent(parent));
        }
        self.refresh_location();

        if let Some(mut children_component) = self.get_mut::<Children>() {
            children_component.0.extend(children.iter().cloned());
        } else {
            self.insert(Children::with(children));
        }

        self
    }

    pub fn insert_child(&mut self, index: usize, child: Entity) -> &mut Self {
        // TODO: What if child already has a parent?
        // TODO: Uniqueness
        self.world.entity_mut(child).insert(Parent(self.entity));
        self.refresh_location();

        if let Some(mut children_component) = self.get_mut::<Children>() {
            children_component.0.insert(index, child);
        } else {
            self.insert(Children::with(&[child]));
        }

        self
    }

    pub fn insert_children(&mut self, index: usize, children: &[Entity]) -> &mut Self {
        // TODO: What if child already has a parent?
        // TODO: Uniqueness
        let parent = self.entity;
        for child in children.iter() {
            self.world.entity_mut(*child).insert(Parent(parent));
        }
        self.refresh_location();

        if let Some(mut children_component) = self.get_mut::<Children>() {
            children_component
                .0
                .splice(index..index, children.iter().cloned());
        } else {
            self.insert(Children::with(children));
        }

        self
    }

    pub fn remove_child(&mut self, child: Entity) -> &mut Self {
        // TODO: Nicer way?
        if let Some(mut children) = self.get_mut::<Children>() {
            let mut found = false;
            children.0.retain(|item| {
                if *item == child {
                    found = true;
                    false
                } else {
                    true
                }
            });
            if found {
                self.world.entity_mut(child).remove::<Parent>();
                self.refresh_location();
            }
        }
        self
    }

    pub fn remove_children(&mut self, children: &[Entity]) -> &mut Self {
        if let Some(mut children_component) = self.get_mut::<Children>() {
            let mut actual_children = Vec::new();
            children_component.0.retain(|item| {
                if children.contains(item) {
                    actual_children.push(*item);
                    false
                } else {
                    true
                }
            });
            for child in &actual_children {
                self.world.entity_mut(*child).remove::<Parent>();
                self.refresh_location();
            }
        }
        self
    }

    pub fn remove_from_parent(&mut self) -> &mut Self {
        if let Some(&Parent(parent_entity)) = self.get::<Parent>() {
            let child = self.entity;
            let mut parent = self.world.entity_mut(parent_entity);
            let mut parent_children = parent.get_mut::<Children>().unwrap();
            parent_children.0.retain(|item| *item != child);
            self.remove::<Parent>();
        }
        self
    }

    fn refresh_location(&mut self) {
        self.location = self.world.entities.get(self.entity).unwrap();
    }
}

fn despawn_recursive(world: &mut World, entity: Entity) {
    if let Some(mut children) = world.entity_mut(entity).get_mut::<Children>() {
        for child in std::mem::take(&mut children.0) {
            despawn_recursive(world, child);
        }
    }
    despawn_self(world, entity);
}

fn despawn_self(world: &mut World, entity: Entity) {
    let location = world.entities.free(entity).unwrap();

    let archetype = &mut world.archetypes[location.archetype_id];
    let remove_result = archetype.swap_remove(location.index);
    if let Some(swapped_entity) = remove_result {
        world.entities.update_location(swapped_entity, location);
    }

    let table_row = location.index;
    unsafe { world.storages.tables[archetype.table_id()].swap_remove_unchecked(table_row) };

    let removed_components = &mut world.removed_components;
    for &component_id in archetype.components() {
        removed_components
            .entry(component_id)
            .or_insert_with(Vec::new)
            .push(entity);
    }
}

unsafe fn get_insert_bundle_info(
    entities: &mut Entities,
    archetypes: &mut Archetypes,
    components: &mut Components,
    storages: &mut Storages,
    bundle_info: &BundleInfo,
    current_location: EntityLocation,
    entity: Entity,
) -> EntityLocation {
    let current_archetype_id = current_location.archetype_id;
    let new_archetype_id = add_bundle_to_archetype(
        archetypes,
        storages,
        components,
        current_archetype_id,
        bundle_info,
    );

    if new_archetype_id == current_archetype_id {
        current_location
    } else {
        let old_archetype = &mut archetypes[current_archetype_id];
        let result = old_archetype.swap_remove(current_location.index);
        if let Some(swapped_entity) = result {
            entities.update_location(swapped_entity, current_location)
        }

        let old_table_id = old_archetype.table_id();
        let new_table_id = archetypes[new_archetype_id].table_id();
        let (old_table, new_table) = storages.tables.get_2_mut(old_table_id, new_table_id);
        old_table.move_to_superset_unchecked(current_location.index, new_table);

        let new_archetype = &mut archetypes[new_archetype_id];
        let new_location = new_archetype.next_location();
        new_archetype.allocate(entity);

        entities.update_location(entity, new_location);

        new_location
    }
}

unsafe fn add_bundle_to_archetype(
    archetypes: &mut Archetypes,
    storages: &mut Storages,
    components: &mut Components,
    archetype_id: ArchetypeId,
    bundle_info: &BundleInfo,
) -> ArchetypeId {
    if let Some(add_bundle) = archetypes[archetype_id]
        .edges()
        .get_add_bundle(bundle_info.id)
    {
        return add_bundle.archetype_id;
    }
    let mut new_components = Vec::new();
    let mut bundle_status = Vec::with_capacity(bundle_info.component_ids.len());

    let current_archetype = &mut archetypes[archetype_id];
    for component_id in bundle_info.component_ids.iter().cloned() {
        if current_archetype.contains(component_id) {
            bundle_status.push(ComponentStatus::Mutated);
        } else {
            bundle_status.push(ComponentStatus::Added);
            new_components.push(component_id);
        }
    }

    if new_components.is_empty() {
        let edges = current_archetype.edges_mut();
        edges.set_add_bundle(bundle_info.id, archetype_id, bundle_status);
        archetype_id
    } else {
        let current_archetype = &archetypes[archetype_id];
        new_components.extend(current_archetype.components());
        new_components.sort();

        let table_id = storages
            .tables
            .get_id_or_insert(&new_components, components);

        let new_archetype_id = archetypes.get_id_or_insert(table_id, &new_components);

        archetypes[archetype_id].edges_mut().set_add_bundle(
            bundle_info.id,
            new_archetype_id,
            bundle_status,
        );
        new_archetype_id
    }
}

fn remove_bundle_from_archetype(
    archetypes: &mut Archetypes,
    storages: &mut Storages,
    components: &mut Components,
    archetype_id: ArchetypeId,
    bundle_info: &BundleInfo,
    intersection: bool,
) -> Option<ArchetypeId> {
    let bundle_id = bundle_info.id;
    let current_archetype = &mut archetypes[archetype_id];
    let remove_bundle_archetype =
        current_archetype.get_remove_bundle_archetype(bundle_id, intersection);

    if let Some(result) = remove_bundle_archetype {
        return result;
    }

    let mut removed_components = Vec::new();
    for component_id in bundle_info.component_ids.iter().cloned() {
        if current_archetype.contains(component_id) {
            removed_components.push(component_id);
        } else if !intersection {
            current_archetype.set_remove_bundle_archetype(bundle_id, None, false);
            return None;
        }
    }

    removed_components.sort();
    let mut next_components = current_archetype.components().to_vec();
    sorted_remove(&mut next_components, &removed_components);

    let next_table_id = if removed_components.is_empty() {
        current_archetype.table_id()
    } else {
        unsafe {
            storages
                .tables
                .get_id_or_insert(&next_components, components)
        }
    };

    let new_archetype_id = Some(archetypes.get_id_or_insert(next_table_id, &next_components));
    archetypes[archetype_id].set_remove_bundle_archetype(bundle_id, new_archetype_id, intersection);

    new_archetype_id
}

fn sorted_remove<T: Eq + Ord + Copy>(source: &mut Vec<T>, remove: &[T]) {
    let mut remove_index = 0;
    source.retain(|value| {
        while remove_index < remove.len() && *value > remove[remove_index] {
            remove_index += 1;
        }

        if remove_index < remove.len() {
            *value != remove[remove_index]
        } else {
            true
        }
    })
}

fn move_entity_after_remove(
    drop: bool,
    entity: Entity,
    old_location: EntityLocation,
    new_archetype_id: ArchetypeId,
    archetypes: &mut Archetypes,
    storages: &mut Storages,
    entities: &mut Entities,
) -> EntityLocation {
    let old_archetype = &mut archetypes[old_location.archetype_id];
    let remove_result = old_archetype.swap_remove(old_location.index);
    if let Some(swapped_entity) = remove_result {
        entities.update_location(swapped_entity, old_location);
    }
    let old_table_row = old_location.index;
    let old_table_id = old_archetype.table_id();

    let new_archetype = &mut archetypes[new_archetype_id];
    let (old_table, new_table) = storages
        .tables
        .get_2_mut(old_table_id, new_archetype.table_id());

    unsafe {
        if drop {
            old_table.move_to_and_drop_missing_unchecked(old_table_row, new_table);
        } else {
            old_table.move_to_and_forget_missing_unchecked(old_table_row, new_table);
        }
    };

    let new_location = new_archetype.next_location();
    new_archetype.allocate(entity);

    entities.update_location(entity, new_location);

    new_location
}

#[cfg(test)]
mod test {
    use crate::ecs::{Entity, World};

    fn check_parent(world: &World, entity: Entity, parent: Option<Entity>) {
        assert_eq!(world.entity(entity).parent(), parent);
    }

    fn check_children(world: &World, entity: Entity, children: Option<&[Entity]>) {
        assert_eq!(world.entity(entity).children(), children);
    }

    #[test]
    fn despawn() {
        let mut world = World::new();
        let entity = world.spawn().id();
        let entity_ref = world.entity_mut(entity);
        entity_ref.despawn();
        assert!(!world.has_entity(entity));
    }

    #[test]
    fn despawn_recursive() {
        let mut world = World::new();
        let child1_entity = world.spawn().id();
        let child2_entity = world.spawn().id();
        let no_parent_entity = world.spawn().id();
        let parent_entity = world
            .spawn()
            .push_children(&[child1_entity, child2_entity])
            .id();
        let grandparent_entity = world.spawn().push_child(parent_entity).id();
        assert!(world.has_entity(child1_entity));
        assert!(world.has_entity(child2_entity));
        assert!(world.has_entity(parent_entity));
        assert!(world.has_entity(grandparent_entity));
        assert!(world.has_entity(no_parent_entity));
        check_children(&world, parent_entity, Some(&[child1_entity, child2_entity]));
        check_children(&world, grandparent_entity, Some(&[parent_entity]));

        world.entity_mut(parent_entity).despawn_recursive();
        assert!(!world.has_entity(child1_entity));
        assert!(!world.has_entity(child2_entity));
        assert!(!world.has_entity(parent_entity));
        assert!(world.has_entity(grandparent_entity));
        assert!(world.has_entity(no_parent_entity));
        check_children(&world, grandparent_entity, Some(&[]));
    }

    #[test]
    fn push_child() {
        let mut world = World::new();

        let grandchild_entity = world.spawn().id();
        let child1_entity = world.spawn().push_child(grandchild_entity).id();
        let child2_entity = world.spawn().id();
        let parent_entity = world
            .spawn()
            .push_child(child1_entity)
            .push_child(child2_entity)
            .id();

        check_children(&world, grandchild_entity, None);
        check_parent(&world, grandchild_entity, Some(child1_entity));
        check_children(&world, child1_entity, Some(&[grandchild_entity]));
        check_parent(&world, child1_entity, Some(parent_entity));
        check_children(&world, child2_entity, None);
        check_parent(&world, child2_entity, Some(parent_entity));
        check_children(&world, parent_entity, Some(&[child1_entity, child2_entity]));
        check_parent(&world, parent_entity, None);
    }

    #[test]
    fn push_children() {
        let mut world = World::new();

        let grandchild_entity = world.spawn().id();
        let child1_entity = world.spawn().push_children(&[grandchild_entity]).id();
        let child2_entity = world.spawn().id();
        let parent_entity = world
            .spawn()
            .push_children(&[child1_entity, child2_entity])
            .id();

        check_children(&world, grandchild_entity, None);
        check_parent(&world, grandchild_entity, Some(child1_entity));
        check_children(&world, child1_entity, Some(&[grandchild_entity]));
        check_parent(&world, child1_entity, Some(parent_entity));
        check_children(&world, child2_entity, None);
        check_parent(&world, child2_entity, Some(parent_entity));
        check_children(&world, parent_entity, Some(&[child1_entity, child2_entity]));
        check_parent(&world, parent_entity, None);
    }

    #[test]
    fn insert_child() {
        let mut world = World::new();

        let child1_entity = world.spawn().id();
        let child2_entity = world.spawn().id();
        let child3_entity = world.spawn().id();
        let parent_entity = world.spawn().id();

        world
            .entity_mut(parent_entity)
            .insert_child(0, child1_entity);

        check_parent(&world, child1_entity, Some(parent_entity));
        check_children(&world, parent_entity, Some(&[child1_entity]));

        world
            .entity_mut(parent_entity)
            .insert_child(1, child3_entity);
        check_parent(&world, child3_entity, Some(parent_entity));
        check_children(&world, parent_entity, Some(&[child1_entity, child3_entity]));

        world
            .entity_mut(parent_entity)
            .insert_child(1, child2_entity);
        check_parent(&world, child2_entity, Some(parent_entity));
        check_children(
            &world,
            parent_entity,
            Some(&[child1_entity, child2_entity, child3_entity]),
        );
    }

    #[test]
    fn insert_children() {
        let mut world = World::new();

        let child1_entity = world.spawn().id();
        let child2_entity = world.spawn().id();
        let child3_entity = world.spawn().id();
        let child4_entity = world.spawn().id();
        let parent_entity = world.spawn().id();

        world
            .entity_mut(parent_entity)
            .insert_children(0, &[child1_entity, child4_entity]);
        check_parent(&world, child1_entity, Some(parent_entity));
        check_parent(&world, child4_entity, Some(parent_entity));
        check_children(&world, parent_entity, Some(&[child1_entity, child4_entity]));

        world
            .entity_mut(parent_entity)
            .insert_children(1, &[child2_entity, child3_entity]);
        check_parent(&world, child2_entity, Some(parent_entity));
        check_parent(&world, child3_entity, Some(parent_entity));
        check_children(
            &world,
            parent_entity,
            Some(&[child1_entity, child2_entity, child3_entity, child4_entity]),
        );
    }

    #[test]
    fn remove_child() {
        let mut world = World::new();

        let child1_entity = world.spawn().id();
        let child2_entity = world.spawn().id();
        let parent_entity = world
            .spawn()
            .push_children(&[child1_entity, child2_entity])
            .id();

        check_children(&world, parent_entity, Some(&[child1_entity, child2_entity]));
        check_parent(&world, child1_entity, Some(parent_entity));
        check_parent(&world, child2_entity, Some(parent_entity));

        world.entity_mut(parent_entity).remove_child(child1_entity);
        check_children(&world, parent_entity, Some(&[child2_entity]));
        check_parent(&world, child1_entity, None);
        check_parent(&world, child2_entity, Some(parent_entity));

        world.entity_mut(parent_entity).remove_child(child2_entity);
        check_children(&world, parent_entity, Some(&[]));
        check_parent(&world, child1_entity, None);
        check_parent(&world, child2_entity, None);
    }

    #[test]
    fn remove_children() {
        let mut world = World::new();

        let child1_entity = world.spawn().id();
        let child2_entity = world.spawn().id();
        let child3_entity = world.spawn().id();
        let parent_entity = world
            .spawn()
            .push_children(&[child1_entity, child2_entity, child3_entity])
            .id();

        check_children(
            &world,
            parent_entity,
            Some(&[child1_entity, child2_entity, child3_entity]),
        );
        check_parent(&world, child1_entity, Some(parent_entity));
        check_parent(&world, child2_entity, Some(parent_entity));
        check_parent(&world, child3_entity, Some(parent_entity));

        world
            .entity_mut(parent_entity)
            .remove_children(&[child2_entity]);
        check_children(&world, parent_entity, Some(&[child1_entity, child3_entity]));
        check_parent(&world, child1_entity, Some(parent_entity));
        check_parent(&world, child2_entity, None);
        check_parent(&world, child3_entity, Some(parent_entity));

        world
            .entity_mut(parent_entity)
            .remove_children(&[child1_entity, child3_entity]);
        check_children(&world, parent_entity, Some(&[]));
        check_parent(&world, child1_entity, None);
        check_parent(&world, child2_entity, None);
        check_parent(&world, child3_entity, None);
    }

    #[test]
    fn remove_from_parent() {
        let mut world = World::new();

        let child1_entity = world.spawn().id();
        let child2_entity = world.spawn().id();
        let parent_entity = world
            .spawn()
            .push_children(&[child1_entity, child2_entity])
            .id();

        check_children(&world, parent_entity, Some(&[child1_entity, child2_entity]));
        check_parent(&world, child1_entity, Some(parent_entity));
        check_parent(&world, child2_entity, Some(parent_entity));

        world.entity_mut(child1_entity).remove_from_parent();
        check_children(&world, parent_entity, Some(&[child2_entity]));
        check_parent(&world, child1_entity, None);
        check_parent(&world, child2_entity, Some(parent_entity));

        world.entity_mut(child2_entity).remove_from_parent();
        check_children(&world, parent_entity, Some(&[]));
        check_parent(&world, child1_entity, None);
        check_parent(&world, child2_entity, None);
    }
}
