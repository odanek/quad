use super::archetype::ArchetypeId;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Entity {
    pub(crate) generation: u32, // TODO: Combine into single u64?
    pub(crate) id: u32,
}

impl Entity {
    #[inline]
    pub fn new(id: u32) -> Self {
        Self { id, generation: 0 }
    }

    #[inline]
    pub fn id(self) -> u32 {
        self.id
    }

    #[inline]
    pub fn generation(self) -> u32 {
        self.generation
    }
}

#[derive(Default)]
pub struct Entities {
    entries: Vec<EntityEntry>,
    free: Vec<u32>, // TODO: use the same pattern as in Arena or Bevy
    len: u32,
}

#[derive(Copy, Clone, Debug)]
struct EntityEntry {
    pub generation: u32,
    pub location: EntityLocation,
}

#[derive(Copy, Clone, Debug)]
pub struct EntityLocation {
    pub archetype_id: ArchetypeId,
    pub index: usize,
}

impl Entities {
    pub fn alloc(&mut self, location: EntityLocation) -> Entity {
        if let Some(id) = self.free.pop() {
            self.len += 1;
            let entry = &mut self.entries[id as usize];
            entry.location = location;

            Entity {
                generation: entry.generation,
                id,
            }
        } else {
            let id = self.len;
            self.len = self.len.checked_add(1).expect("Too many entities");
            let entry = EntityEntry {
                generation: 0,
                location,
            };
            self.entries.push(entry);
            Entity::new(id)
        }
    }

    pub fn free(&mut self, entity: Entity) -> Option<EntityLocation> {
        let entry = &mut self.entries[entity.id as usize];
        if entry.generation != entity.generation {
            return None;
        }
        entry.generation += 1;

        let location = entry.location; // TODO: Reset location to some empty value?

        self.free.push(entity.id);
        self.len -= 1;
        Some(location)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.free.clear();
        self.len = 0;
    }

    pub fn get(&self, entity: Entity) -> Option<EntityLocation> {
        if (entity.id as usize) < self.entries.len() {
            let entry = &self.entries[entity.id as usize];
            if entry.generation != entity.generation {
                return None;
            }
            Some(entry.location)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityLocation> {
        if (entity.id as usize) < self.entries.len() {
            let entry = &mut self.entries[entity.id as usize];
            if entry.generation != entity.generation {
                return None;
            }
            Some(&mut entry.location)
        } else {
            None
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entries
            .get(entity.id as usize)
            .map_or(true, |entry| entry.generation == entity.generation)
    }

    pub(crate) fn update_location(&mut self, entity: Entity, location: EntityLocation) {
        self.entries[entity.id as usize].location = location;
    }
}
