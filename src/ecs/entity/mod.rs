#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Entity {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

#[derive(Default)]
pub struct Entities {
    entries: Vec<EntityEntry>,
    free_cursor: Option<usize>,
    len: usize,
}

// TODO Use enum as in Arena
struct EntityEntry {
    pub generation: u32,
    pub location: EntityLocation,
}

pub struct EntityLocation {
//     // archetype, index
}

impl Entities {
    pub fn get(&self, entity: Entity) -> Option<EntityLocation> {
        None
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityLocation> {
        None
    }

    pub fn contains(&self, entity: Entity) -> bool {
        false
    }
}

