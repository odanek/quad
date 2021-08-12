use std::marker::PhantomData;

use crate::ecs::{
    component::{bundle::Bundle, Component},
    entity::Entities,
    Entity, World,
};

use super::{
    function_system::SystemMeta,
    system_param::{SystemParam, SystemParamFetch, SystemParamState},
};

#[derive(Default)]
pub struct CommandQueue {
    commands: Vec<Box<dyn Command>>,
}

unsafe impl Send for CommandQueue {}

unsafe impl Sync for CommandQueue {}

impl CommandQueue {
    #[inline]
    pub fn push<C>(&mut self, command: C)
    where
        C: Command,
    {
        self.commands.push(Box::new(command));
    }

    #[inline]
    pub fn apply(&mut self, world: &mut World) {
        world.flush();
        for command in self.commands.drain(0..) {
            command.write(world);
        }
    }
}

pub trait Command: Send + Sync + 'static {
    fn write(self: Box<Self>, world: &mut World);
}

pub struct Commands<'a> {
    queue: &'a mut CommandQueue,
    entities: &'a Entities,
}

impl<'a> Commands<'a> {
    pub fn new(queue: &'a mut CommandQueue, world: &'a World) -> Self {
        Self {
            queue,
            entities: world.entities(),
        }
    }

    pub fn spawn(&mut self) -> EntityCommands<'a, '_> {
        let entity = self.entities.reserve_entity();
        EntityCommands {
            entity,
            commands: self,
        }
    }

    pub fn spawn_bundle<'b, T: Bundle>(&'b mut self, bundle: T) -> EntityCommands<'a, 'b> {
        let mut entity = self.spawn();
        entity.insert_bundle(bundle);
        entity
    }

    pub fn entity(&mut self, entity: Entity) -> EntityCommands<'a, '_> {
        EntityCommands {
            entity,
            commands: self,
        }
    }

    pub fn insert_resource<T: Component>(&mut self, resource: T) {
        self.queue.push(InsertResource { resource })
    }

    pub fn remove_resource<T: Component>(&mut self) {
        self.queue.push(RemoveResource::<T> {
            phantom: PhantomData,
        });
    }

    pub fn add<C: Command>(&mut self, command: C) {
        self.queue.push(command);
    }
}

pub struct EntityCommands<'a, 'b> {
    entity: Entity,
    commands: &'b mut Commands<'a>,
}

impl<'a, 'b> EntityCommands<'a, 'b> {
    #[inline]
    pub fn id(&self) -> Entity {
        self.entity
    }

    pub fn insert_bundle(&mut self, bundle: impl Bundle) -> &mut Self {
        self.commands.add(InsertBundle {
            entity: self.entity,
            bundle,
        });
        self
    }

    pub fn insert(&mut self, component: impl Component) -> &mut Self {
        self.commands.add(Insert {
            entity: self.entity,
            component,
        });
        self
    }

    pub fn remove_bundle<T>(&mut self) -> &mut Self
    where
        T: Bundle,
    {
        self.commands.add(RemoveBundle::<T> {
            entity: self.entity,
            phantom: PhantomData,
        });
        self
    }

    pub fn remove<T>(&mut self) -> &mut Self
    where
        T: Component,
    {
        self.commands.add(Remove::<T> {
            entity: self.entity,
            phantom: PhantomData,
        });
        self
    }

    pub fn despawn(&mut self) {
        self.commands.add(Despawn {
            entity: self.entity,
        })
    }

    pub fn commands(&mut self) -> &mut Commands<'a> {
        self.commands
    }
}

#[derive(Debug)]
pub struct Despawn {
    pub entity: Entity,
}

impl Command for Despawn {
    fn write(self: Box<Self>, world: &mut World) {
        if !world.despawn(self.entity) {
            panic!("Failed to despawn non-existent entity {:?}", self.entity);
        }
    }
}

pub struct InsertBundle<T> {
    pub entity: Entity,
    pub bundle: T,
}

impl<T> Command for InsertBundle<T>
where
    T: Bundle + 'static,
{
    fn write(self: Box<Self>, world: &mut World) {
        world.entity_mut(self.entity).insert_bundle(self.bundle);
    }
}

#[derive(Debug)]
pub struct Insert<T> {
    pub entity: Entity,
    pub component: T,
}

impl<T> Command for Insert<T>
where
    T: Component,
{
    fn write(self: Box<Self>, world: &mut World) {
        world.entity_mut(self.entity).insert(self.component);
    }
}

#[derive(Debug)]
pub struct Remove<T> {
    pub entity: Entity,
    pub phantom: PhantomData<T>,
}

impl<T> Command for Remove<T>
where
    T: Component,
{
    fn write(self: Box<Self>, world: &mut World) {
        if let Some(mut entity_mut) = world.get_entity_mut(self.entity) {
            entity_mut.remove::<T>();
        }
    }
}

#[derive(Debug)]
pub struct RemoveBundle<T> {
    pub entity: Entity,
    pub phantom: PhantomData<T>,
}

impl<T> Command for RemoveBundle<T>
where
    T: Bundle,
{
    fn write(self: Box<Self>, world: &mut World) {
        if let Some(mut entity_mut) = world.get_entity_mut(self.entity) {
            entity_mut.remove_bundle_intersection::<T>();
        }
    }
}

pub struct InsertResource<T: Component> {
    pub resource: T,
}

impl<T: Component> Command for InsertResource<T> {
    fn write(self: Box<Self>, world: &mut World) {
        world.insert_resource(self.resource);
    }
}

pub struct RemoveResource<T: Component> {
    pub phantom: PhantomData<T>,
}

impl<T: Component> Command for RemoveResource<T> {
    fn write(self: Box<Self>, world: &mut World) {
        world.remove_resource::<T>();
    }
}

impl<'a> SystemParam for Commands<'a> {
    type Fetch = CommandQueue;
}

impl SystemParamState for CommandQueue {
    fn new(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
        Default::default()
    }

    fn apply(&mut self, world: &mut World) {
        self.apply(world);
    }
}

impl<'a> SystemParamFetch<'a> for CommandQueue {
    type Item = Commands<'a>;

    #[inline]
    unsafe fn get_param(
        state: &'a mut Self,
        _system_meta: &SystemMeta,
        world: &'a World,
    ) -> Self::Item {
        Commands::new(state, world)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct SpawnCommand;

    impl Command for SpawnCommand {
        fn write(self: Box<Self>, world: &mut World) {
            world.spawn();
        }
    }

    #[test]
    fn test_command_queue_inner() {
        let mut queue = CommandQueue::default();

        queue.push(SpawnCommand);
        queue.push(SpawnCommand);

        let mut world = World::new();
        queue.apply(&mut world);

        assert_eq!(world.entities().len(), 2);

        queue.apply(&mut world);
        assert_eq!(world.entities().len(), 2);
    }
}
