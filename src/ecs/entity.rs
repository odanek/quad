mod archetype;

use std::convert::TryFrom;
use std::sync::atomic::{AtomicI64, Ordering};

pub use archetype::{Archetype, ArchetypeGeneration, ArchetypeId, Archetypes};

pub enum AllocAtWithoutReplacement {
    Exists(EntityLocation),
    DidNotExist,
    ExistsWithWrongGeneration,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct Entity {
    pub(crate) generation: u32,
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
    pending: Vec<u32>,
    free_cursor: AtomicI64,
    len: u32,
}

#[derive(Copy, Clone, Debug)]
struct EntityMeta {
    pub generation: u32,
    pub location: EntityLocation,
}

impl EntityMeta {
    // TODO: This is ugly and not safe
    const EMPTY: EntityMeta = EntityMeta {
        generation: 0,
        location: EntityLocation::EMPTY,
    };
}

#[derive(Copy, Clone, Debug)]
pub struct EntityLocation {
    pub archetype_id: ArchetypeId,
    pub index: usize,
}

impl EntityLocation {
    // TODO: This is ugly and not safe
    const EMPTY: EntityLocation = EntityLocation {
        archetype_id: ArchetypeId::INVALID,
        index: usize::MAX,
    };
}

#[allow(dead_code)]
impl Entities {
    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    pub fn meta_len(&self) -> usize {
        self.meta.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn alloc(&mut self) -> Entity {
        self.verify_flushed();
        self.len += 1;
        if let Some(id) = self.pending.pop() {
            let new_free_cursor = self.pending.len() as i64;
            *self.free_cursor.get_mut() = new_free_cursor;
            Entity {
                generation: self.meta[id as usize].generation,
                id,
            }
        } else {
            let id = u32::try_from(self.meta.len()).expect("Too many entities");
            self.meta.push(EntityMeta::EMPTY);
            Entity { generation: 0, id }
        }
    }

    pub fn alloc_at_without_replacement(&mut self, entity: Entity) -> AllocAtWithoutReplacement {
        self.verify_flushed();

        let result = if entity.id as usize >= self.meta.len() {
            self.pending.extend((self.meta.len() as u32)..entity.id);
            let new_free_cursor = self.pending.len() as i64;
            *self.free_cursor.get_mut() = new_free_cursor;
            self.meta.resize(entity.id as usize + 1, EntityMeta::EMPTY);
            self.len += 1;
            AllocAtWithoutReplacement::DidNotExist
        } else if let Some(index) = self.pending.iter().position(|item| *item == entity.id) {
            self.pending.swap_remove(index);
            let new_free_cursor = self.pending.len() as i64;
            *self.free_cursor.get_mut() = new_free_cursor;
            self.len += 1;
            AllocAtWithoutReplacement::DidNotExist
        } else {
            let current_meta = &mut self.meta[entity.id as usize];
            if current_meta.location.archetype_id == ArchetypeId::INVALID {
                AllocAtWithoutReplacement::DidNotExist
            } else if current_meta.generation == entity.generation {
                AllocAtWithoutReplacement::Exists(current_meta.location)
            } else {
                return AllocAtWithoutReplacement::ExistsWithWrongGeneration;
            }
        };

        self.meta[entity.id as usize].generation = entity.generation;
        result
    }

    pub fn free(&mut self, entity: Entity) -> Option<EntityLocation> {
        self.verify_flushed();

        let index = entity.id as usize;
        if index >= self.meta.len() {
            return None;
        }

        let meta = &mut self.meta[index];
        if meta.generation != entity.generation {
            return None;
        }
        meta.generation += 1;

        self.pending.push(entity.id);
        *self.free_cursor.get_mut() = self.pending.len() as i64;

        self.len -= 1;

        Some(meta.location)
    }

    pub fn reserve_entity(&self) -> Entity {
        let n = self.free_cursor.fetch_sub(1, Ordering::Relaxed);
        if n > 0 {
            let id = self.pending[(n - 1) as usize];
            Entity {
                generation: self.meta[id as usize].generation,
                id,
            }
        } else {
            let id = u32::try_from(self.meta.len() as i64 - n).expect("Too many entities");
            Entity { generation: 0, id }
        }
    }

    pub fn reserve_entities(&self, count: u32) {
        // Use one atomic subtract to grab a range of new IDs. The range might be
        // entirely nonnegative, meaning all IDs come from the freelist, or entirely
        // negative, meaning they are all new IDs to allocate, or a mix of both.
        self.free_cursor.fetch_sub(count as i64, Ordering::Relaxed);
    }

    // This resets the generation of existing meta so some invalid Entity objects may become valid again in the future
    pub fn clear(&mut self) {
        self.meta.clear();
        self.pending.clear();
        *self.free_cursor.get_mut() = 0;
        self.len = 0;
    }

    pub fn has(&self, entity: Entity) -> bool {
        let index = entity.id as usize;
        index < self.meta.len() && self.meta[index].generation == entity.generation
    }

    pub fn get(&self, entity: Entity) -> Option<EntityLocation> {
        if (entity.id as usize) < self.meta.len() {
            let meta = &self.meta[entity.id as usize];
            if meta.generation != entity.generation
                || meta.location.archetype_id == ArchetypeId::INVALID
            {
                return None;
            }
            Some(meta.location)
        } else {
            None
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.meta
            .get(entity.id as usize)
            .map_or(true, |meta| meta.generation == entity.generation)
    }

    pub fn update_location(&mut self, entity: Entity, location: EntityLocation) {
        self.meta[entity.id as usize].location = location;
    }

    pub unsafe fn flush(&mut self, mut init: impl FnMut(Entity) -> EntityLocation) {
        let free_cursor = self.free_cursor.get_mut();
        let mut current_free_cursor = *free_cursor;

        if current_free_cursor < 0 {
            let meta = &mut self.meta;
            let mut id = meta.len() as u32;
            while current_free_cursor < 0 {
                let entity = Entity::new(id);
                let location = init(entity);
                meta.push(EntityMeta {
                    generation: entity.generation,
                    location,
                });
                id += 1;
                current_free_cursor += 1;
                self.len += 1;
            }
            *free_cursor = 0;
        }

        let pending_index = current_free_cursor as usize;
        for id in self.pending.drain(pending_index..) {
            let meta = &mut self.meta[id as usize];
            let entity = Entity {
                id,
                generation: meta.generation,
            };
            meta.location = init(entity);
            self.len += 1;
        }
    }

    pub fn flush_as_invalid(&mut self) {
        unsafe {
            self.flush(|_entity| EntityLocation::EMPTY);
        }
    }

    fn verify_flushed(&mut self) {
        debug_assert!(
            !self.needs_flush(),
            "flush() needs to be called before this operation is legal"
        );
    }

    fn needs_flush(&mut self) -> bool {
        *self.free_cursor.get_mut() != self.pending.len() as i64
    }
}
