use std::hash::Hash;

pub trait StageLabel: Eq + PartialEq + Hash {
    fn id(&self) -> u32;
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum MainStage {
    LoadAssets = 0,
    PreUpdate = 1,
    PostUpdate = 2,
    AssetEvents = 3,
    Flush = 4,
}

impl StageLabel for MainStage {
    fn id(&self) -> u32 {
        *self as u32
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum RenderStage {
    Extract = 10,
    Prepare = 11,
    Queue = 12,
    PhaseSort = 13,
    Render = 14,
    Cleanup = 15,
}

impl StageLabel for RenderStage {
    fn id(&self) -> u32 {
        *self as u32
    }
}
