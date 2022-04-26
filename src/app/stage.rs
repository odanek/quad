use std::hash::Hash;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct StageId(u32);

pub trait StageLabel: Eq + PartialEq + Hash {
    fn id(&self) -> StageId;
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum MainStage {
    LoadAssets = 0,
    PreUpdate = 1,
    PreTransformUpdate = 2,
    TransformUpdate = 3,
    PostTransformUpdate = 4,
    AssetEvents = 5,
    Flush = 6,
}

impl StageLabel for MainStage {
    fn id(&self) -> StageId {
        StageId(*self as u32)
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
    fn id(&self) -> StageId {
        StageId(*self as u32)
    }
}
