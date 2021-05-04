use std::{alloc::Layout, any::TypeId, collections::HashMap};

pub trait Component: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Component for T {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StorageType {
    Table,
}

impl Default for StorageType {
    fn default() -> Self {
        StorageType::Table
    }
}

#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct ComponentId(u32);

impl ComponentId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug)]
pub struct ComponentInfo {
    name: String,
    id: ComponentId,
    type_id: TypeId,
    layout: Layout,
    drop: unsafe fn(*mut u8),
    storage_type: StorageType,
}

#[derive(Debug, Default)]
pub struct Components {
    components: Vec<ComponentInfo>,
    indices: HashMap<TypeId, usize>,
}
