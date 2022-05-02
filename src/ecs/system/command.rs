use std::marker::PhantomData;

use crate::ecs::{
    component::{Bundle, Component, Tick},
    entity::Entities,
    Entity, Resource, World,
};

use super::{
    function_system::SystemMeta,
    system_param::{ReadOnlySystemParamFetch, SystemParam, SystemParamFetch, SystemParamState},
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

pub struct Commands<'w, 's> {
    queue: &'s mut CommandQueue,
    entities: &'w Entities,
}

impl<'w, 's> Commands<'w, 's> {
    pub fn new(queue: &'s mut CommandQueue, world: &'w World) -> Self {
        Self {
            queue,
            entities: world.entities(),
        }
    }

    pub fn spawn(&mut self) -> EntityCommands<'w, 's, '_> {
        let entity = self.entities.reserve_entity();
        EntityCommands {
            entity,
            commands: self,
        }
    }

    pub(crate) fn get_or_spawn<'a>(&'a mut self, entity: Entity) -> EntityCommands<'w, 's, 'a> {
        self.add(GetOrSpawn { entity });
        EntityCommands {
            entity,
            commands: self,
        }
    }

    pub fn spawn_bundle<T: Bundle>(&mut self, bundle: T) -> EntityCommands<'w, 's, '_> {
        let mut entity = self.spawn();
        entity.insert_bundle(bundle);
        entity
    }

    pub fn insert_or_spawn_batch<I, B>(&mut self, bundles_iter: I)
    where
        I: IntoIterator + Send + Sync + 'static,
        I::IntoIter: Iterator<Item = (Entity, B)>,
        B: Bundle,
    {
        self.queue.push(InsertOrSpawnBatch { bundles_iter });
    }

    pub fn entity(&mut self, entity: Entity) -> EntityCommands<'w, 's, '_> {
        EntityCommands {
            entity,
            commands: self,
        }
    }

    pub fn init_resource<T: Resource + Default>(&mut self) {
        self.queue.push(InsertResource {
            resource: T::default(),
        })
    }

    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.queue.push(InsertResource { resource })
    }

    pub fn remove_resource<T: Resource>(&mut self) {
        self.queue.push(RemoveResource::<T> {
            phantom: PhantomData,
        });
    }

    fn add<C: Command>(&mut self, command: C) {
        self.queue.push(command);
    }
}

pub struct EntityCommands<'w, 's, 'a> {
    entity: Entity,
    commands: &'a mut Commands<'w, 's>,
}

impl<'w, 's, 'a> EntityCommands<'w, 's, 'a> {
    #[inline]
    pub fn id(&self) -> Entity {
        self.entity
    }

    #[inline]
    pub(crate) fn commands(&mut self) -> &mut Commands<'w, 's> {
        self.commands
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

    pub fn despawn_recursive(&mut self) {
        self.commands.add(DespawnRecursive {
            entity: self.entity,
        })
    }

    pub fn with_children(&mut self, spawn_children: impl FnOnce(&mut ChildBuilder)) -> &mut Self {
        let parent = self.id();
        let push_children = {
            let mut builder = ChildBuilder {
                commands: self.commands(),
                push_children: PushChildren {
                    children: Vec::with_capacity(8),
                    parent,
                },
            };
            spawn_children(&mut builder);
            builder.push_children
        };

        self.commands().add(push_children);
        self
    }

    pub fn push_child(&mut self, child: Entity) -> &mut Self {
        let parent = self.id();
        self.commands().add(PushChild { child, parent });
        self
    }

    pub fn push_children(&mut self, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        self.commands().add(PushChildren {
            children: children.to_vec(),
            parent,
        });
        self
    }

    pub fn insert_child(&mut self, index: usize, child: Entity) -> &mut Self {
        let parent = self.id();
        self.commands().add(InsertChild {
            child,
            index,
            parent,
        });
        self
    }

    pub fn insert_children(&mut self, index: usize, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        self.commands().add(InsertChildren {
            children: children.to_vec(),
            index,
            parent,
        });
        self
    }

    pub fn remove_child(&mut self, child: Entity) -> &mut Self {
        let parent = self.id();
        self.commands().add(RemoveChild { child, parent });
        self
    }

    pub fn remove_children(&mut self, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        self.commands().add(RemoveChildren {
            children: children.to_vec(),
            parent,
        });
        self
    }

    pub fn remove_from_parent(&mut self, child: Entity) -> &mut Self {
        self.commands().add(RemoveFromParent { child });
        self
    }
}

pub struct ChildBuilder<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    push_children: PushChildren,
}

impl<'w, 's, 'a> ChildBuilder<'w, 's, 'a> {
    pub fn spawn_bundle(&mut self, bundle: impl Bundle) -> EntityCommands<'w, 's, '_> {
        let e = self.commands.spawn_bundle(bundle);
        self.push_children.children.push(e.id());
        e
    }

    pub fn spawn(&mut self) -> EntityCommands<'w, 's, '_> {
        let e = self.commands.spawn();
        self.push_children.children.push(e.id());
        e
    }

    pub fn parent_entity(&self) -> Entity {
        self.push_children.parent
    }

    pub fn add_command<C: Command + 'static>(&mut self, command: C) -> &mut Self {
        self.commands.add(command);
        self
    }
}

#[derive(Debug)]
pub struct GetOrSpawn {
    entity: Entity,
}

impl Command for GetOrSpawn {
    fn write(self: Box<Self>, world: &mut World) {
        world.get_or_spawn(self.entity);
    }
}

pub struct InsertOrSpawnBatch<I, B>
where
    I: IntoIterator + Send + Sync + 'static,
    B: Bundle,
    I::IntoIter: Iterator<Item = (Entity, B)>,
{
    pub bundles_iter: I,
}

impl<I, B> Command for InsertOrSpawnBatch<I, B>
where
    I: IntoIterator + Send + Sync + 'static,
    B: Bundle,
    I::IntoIter: Iterator<Item = (Entity, B)>,
{
    fn write(self: Box<Self>, world: &mut World) {
        world.insert_or_spawn_batch(self.bundles_iter);
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

#[derive(Debug)]
pub struct DespawnRecursive {
    pub entity: Entity,
}

impl Command for DespawnRecursive {
    fn write(self: Box<Self>, world: &mut World) {
        if !world.despawn_recursive(self.entity) {
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

pub struct InsertResource<T: Resource> {
    pub resource: T,
}

impl<T: Resource> Command for InsertResource<T> {
    fn write(self: Box<Self>, world: &mut World) {
        world.insert_resource(self.resource);
    }
}

pub struct RemoveResource<T: Resource> {
    pub phantom: PhantomData<T>,
}

impl<T: Resource> Command for RemoveResource<T> {
    fn write(self: Box<Self>, world: &mut World) {
        world.remove_resource::<T>();
    }
}

pub struct PushChild {
    parent: Entity,
    child: Entity,
}

impl Command for PushChild {
    fn write(self: Box<Self>, world: &mut World) {
        world.entity_mut(self.parent).push_child(self.child);
    }
}

pub struct PushChildren {
    parent: Entity,
    children: Vec<Entity>,
}

impl Command for PushChildren {
    fn write(self: Box<Self>, world: &mut World) {
        world.entity_mut(self.parent).push_children(&self.children);
    }
}

pub struct InsertChild {
    parent: Entity,
    index: usize,
    child: Entity,
}

impl Command for InsertChild {
    fn write(self: Box<Self>, world: &mut World) {
        world
            .entity_mut(self.parent)
            .insert_child(self.index, self.child);
    }
}

pub struct InsertChildren {
    parent: Entity,
    children: Vec<Entity>,
    index: usize,
}

impl Command for InsertChildren {
    fn write(self: Box<Self>, world: &mut World) {
        world
            .entity_mut(self.parent)
            .insert_children(self.index, &self.children);
    }
}

pub struct RemoveChild {
    parent: Entity,
    child: Entity,
}

impl Command for RemoveChild {
    fn write(self: Box<Self>, world: &mut World) {
        world.entity_mut(self.parent).remove_child(self.child);
    }
}

pub struct RemoveChildren {
    parent: Entity,
    children: Vec<Entity>,
}

impl Command for RemoveChildren {
    fn write(self: Box<Self>, world: &mut World) {
        world
            .entity_mut(self.parent)
            .remove_children(&self.children);
    }
}

pub struct RemoveFromParent {
    child: Entity,
}

impl Command for RemoveFromParent {
    fn write(self: Box<Self>, world: &mut World) {
        world.entity_mut(self.child).remove_from_parent();
    }
}

unsafe impl ReadOnlySystemParamFetch for CommandQueue {}

impl<'w, 's> SystemParam for Commands<'w, 's> {
    type Fetch = CommandQueue;
}

unsafe impl SystemParamState for CommandQueue {
    fn new(_world: &mut World, _system_meta: &mut SystemMeta) -> Self {
        Default::default()
    }

    fn apply(&mut self, world: &mut World) {
        self.apply(world);
    }
}

impl<'w, 's> SystemParamFetch<'w, 's> for CommandQueue {
    type Item = Commands<'w, 's>;

    #[inline]
    unsafe fn get_param(
        state: &'s mut Self,
        _system_meta: &SystemMeta,
        world: &'w World,
        _change_tick: Tick,
    ) -> Self::Item {
        Commands::new(state, world)
    }
}

#[cfg(test)]
mod test {
    use crate::transform::{Children, Parent};

    use super::*;
    use quad::ecs::Component;

    struct SpawnCommand;

    impl Command for SpawnCommand {
        fn write(self: Box<Self>, world: &mut World) {
            world.spawn();
        }
    }

    #[derive(Component)]
    struct C(u32);

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

    #[test]
    fn build_children() {
        let mut world = World::default();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let mut children = Vec::new();
        let parent = commands.spawn().insert(C(1)).id();
        commands.entity(parent).with_children(|parent| {
            children.push(parent.spawn().insert(C(2)).id());
            children.push(parent.spawn().insert(C(3)).id());
            children.push(parent.spawn().insert(C(4)).id());
        });

        queue.apply(&mut world);
        assert_eq!(
            world.entity(parent).get::<Children>().unwrap().0.as_slice(),
            children.as_slice(),
        );
        assert_eq!(
            *world.entity(children[0]).get::<Parent>().unwrap(),
            Parent(parent)
        );
        assert_eq!(
            *world.entity(children[1]).get::<Parent>().unwrap(),
            Parent(parent)
        );
    }
}
