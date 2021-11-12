mod archetype;
#[allow(clippy::module_inception)]
mod entity;

pub use archetype::{Archetype, ArchetypeGeneration, ArchetypeId, Archetypes};
pub use entity::{Entities, Entity, EntityLocation};
