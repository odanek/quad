use self::archetype::ArchetypeId;

pub mod archetype;


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
    meta: Vec<EntityMeta>,
    free: Vec<u32>, // TODO: use the same pattern as in Arena or Bevy
    len: u32,
}

#[derive(Copy, Clone, Debug)]
struct EntityMeta {
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
            let meta = &mut self.meta[id as usize];
            meta.location = location;

            Entity {
                generation: meta.generation,
                id,
            }
        } else {
            let id = self.len;
            self.len = self.len.checked_add(1).expect("Too many entities");
            let meta = EntityMeta {
                generation: 0,
                location,
            };
            self.meta.push(meta);
            Entity::new(id)
        }
    }

    pub fn free(&mut self, entity: Entity) -> Option<EntityLocation> {
        let index = entity.id as usize;
        if index >= self.meta.len() {
            return None;
        }
        let meta = &mut self.meta[index];
        if meta.generation != entity.generation {
            return None;
        }
        meta.generation += 1;

        let location = meta.location; // TODO: Reset location to some empty value?

        self.free.push(entity.id);
        self.len -= 1;
        Some(location)
    }

    pub fn clear(&mut self) {
        // TODO: If new entity is added it will have generation 0 again which can make some invalid entity ref valid again
        self.meta.clear();
        self.free.clear();
        self.len = 0;
    }

    pub fn get(&self, entity: Entity) -> Option<EntityLocation> {
        if (entity.id as usize) < self.meta.len() {
            let meta = &self.meta[entity.id as usize];
            if meta.generation != entity.generation {
                return None;
            }
            Some(meta.location)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut EntityLocation> {
        if (entity.id as usize) < self.meta.len() {
            let meta = &mut self.meta[entity.id as usize];
            if meta.generation != entity.generation {
                return None;
            }
            Some(&mut meta.location)
        } else {
            None
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.meta
            .get(entity.id as usize)
            .map_or(true, |meta| meta.generation == entity.generation)
    }

    pub(crate) fn update_location(&mut self, entity: Entity, location: EntityLocation) {
        self.meta[entity.id as usize].location = location;
    }
}
