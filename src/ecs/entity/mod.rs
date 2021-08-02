use std::sync::atomic::{AtomicI64, Ordering};
use std::convert::TryFrom;

use self::archetype::ArchetypeId;

pub mod archetype;

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
        self.verify_flushed();
        
        if let Some(id) = self.pending.pop() {
            *self.free_cursor.get_mut() = self.pending.len() as i64;
            let meta = &mut self.meta[id as usize];
            meta.location = location;

            Entity {
                generation: meta.generation,
                id,
            }
        } else {
            let id = u32::try_from(self.meta.len()).expect("too many entities");
            let meta = EntityMeta {
                generation: 0,
                location,
            };
            self.meta.push(meta);
            Entity::new(id)
        }
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
        Some(meta.location)
    }

    pub fn reserve_entity(&self) -> Entity {
        // TODO: This returns Entity that has valid generation but is in fact invalid until flush is called
        let n = self.free_cursor.fetch_sub(1, Ordering::Relaxed);
        if n > 0 {
            let id = self.pending[(n - 1) as usize];
            Entity {
                generation: self.meta[id as usize].generation,
                id,
            }
        } else {
            let id = u32::try_from(self.meta.len() as i64 - n).expect("too many entities");
            Entity {
                generation: 0,
                id,
            }
        }
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

    pub fn update_location(&mut self, entity: Entity, location: EntityLocation) {
        self.meta[entity.id as usize].location = location;
    }

    pub fn flush(&mut self, mut init: impl FnMut(Entity, &mut EntityLocation)) {
        todo!()
        // let free_cursor = self.free_cursor.get_mut();
        // let current_free_cursor = *free_cursor;

        // let new_free_cursor = if current_free_cursor >= 0 {
        //     current_free_cursor as usize
        // } else {



        //     let old_meta_len = self.meta.len();
        //     let new_meta_len = old_meta_len + (-current_free_cursor as usize);



        //     self.meta.resize(new_meta_len, EntityMeta::EMPTY);
        //     self.len += -current_free_cursor as u32;
        //     for (id, meta) in self.meta.iter_mut().enumerate().skip(old_meta_len) {
        //         init(
        //             Entity {
        //                 id: id as u32,
        //                 generation: meta.generation,
        //             },
        //             &mut meta.location,
        //         );
        //     }

        //     *free_cursor = 0;
        //     0
        // };

        // self.len += (self.pending.len() - new_free_cursor) as u32;
        // for id in self.pending.drain(new_free_cursor..) {
        //     let meta = &mut self.meta[id as usize];
        //     init(
        //         Entity {
        //             id,
        //             generation: meta.generation,
        //         },
        //         &mut meta.location,
        //     );
        // }
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
