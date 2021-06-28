mod entity_ref;

use self::entity_ref::{EntityMut, EntityRef};

use super::{
    archetype::Archetypes,
    bundle::Bundles,
    component::Components,
    resource::{Resource, ResourceId},
    schedule::Executor,
    storage::Storages,
    BoxedSystem, Entities, Entity, IntoSystem, Resources, System,
};

#[derive(Default)]
pub struct World {
    resources: Resources,
    archetypes: Archetypes,
    entities: Entities,
    components: Components,
    storages: Storages,
    bundles: Bundles,
    executor: Executor,
}

impl World {
    #[inline]
    pub fn new() -> World {
        World::default()
    }

    #[inline]
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    #[inline]
    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    #[inline]
    pub fn archetypes(&self) -> &Archetypes {
        &self.archetypes
    }

    #[inline]
    pub fn components(&self) -> &Components {
        &self.components
    }

    #[inline]
    pub fn storages(&self) -> &Storages {
        &self.storages
    }

    #[inline]
    pub fn bundles(&self) -> &Bundles {
        &self.bundles
    }

    #[inline]
    pub fn resource_id<T: Resource>(&self) -> Option<ResourceId> {
        self.resources.get_id::<T>()
    }

    #[inline]
    pub fn add_resource<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.resources.add(resource)
    }

    #[inline]
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resources.remove()
    }

    #[inline]
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources.get()
    }

    #[inline]
    pub fn resource<T: Resource>(&self) -> &T {
        self.get_resource().expect("Resource not found")
    }

    #[inline]
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.get_mut()
    }

    #[inline]
    pub fn resource_mut<T: Resource>(&mut self) -> &mut T {
        self.get_resource_mut().expect("Resource not found")
    }

    pub fn spawn(&mut self) -> EntityMut {
        let archetype = self.archetypes.empty_mut();
        let location = archetype.next_location();
        let entity = self.entities.alloc(location);
        let table = &mut self.storages.tables[archetype.table_id()];
        archetype.allocate(entity);
        unsafe {
            table.allocate();
            EntityMut::new(self, entity, location)
        }
    }

    #[inline]
    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.get_entity_mut(entity)
            .map(|e| {
                e.despawn();
                true
            })
            .unwrap_or(false)
    }

    #[inline]
    pub fn get_entity(&self, entity: Entity) -> Option<EntityRef> {
        let location = self.entities.get(entity)?;
        Some(EntityRef::new(self, entity, location))
    }

    #[inline]
    pub fn entity(&self, entity: Entity) -> EntityRef {
        self.get_entity(entity).expect("Entity does not exist")
    }

    #[inline]
    pub fn get_entity_mut(&mut self, entity: Entity) -> Option<EntityMut> {
        let location = self.entities.get(entity)?;
        Some(EntityMut::new(self, entity, location))
    }

    #[inline]
    pub fn entity_mut(&mut self, entity: Entity) -> EntityMut {
        self.get_entity_mut(entity).expect("Entity does not exist")
    }

    #[inline]
    pub fn system<T, S>(&mut self, system: S) -> T
    where
        T: System,
        S: IntoSystem<T>,
    {
        let mut system = self.executor.system(system);
        system.initialize(self);
        system
    }

    #[inline]
    pub fn run<Out: 'static>(&mut self, system: &mut BoxedSystem<(), Out>) -> Out
    {
        system.run((), self)
    }

    #[inline]
    pub fn run_with<In: 'static, Out: 'static>(&mut self, system: &mut BoxedSystem<In, Out>, param: In) -> Out {
        system.run(param, self)
    }
}
