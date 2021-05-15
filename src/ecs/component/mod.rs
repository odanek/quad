pub mod type_info;

use std::{
    alloc::Layout,
    any::{TypeId},
    collections::{hash_map::Entry, HashMap},
};

use self::type_info::TypeInfo;

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
pub struct ComponentId(usize);

impl ComponentId {
    #[inline]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub struct ComponentInfo {
    name: &'static str,
    id: ComponentId,
    type_id: TypeId,
    layout: Layout,
    drop: unsafe fn(*mut u8),
    storage_type: StorageType,
}

impl ComponentInfo {
    #[inline]
    pub fn id(&self) -> ComponentId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        self.layout
    }

    #[inline]
    pub fn drop(&self) -> unsafe fn(*mut u8) {
        self.drop
    }

    #[inline]
    pub fn storage_type(&self) -> StorageType {
        self.storage_type
    }

    pub fn new(id: ComponentId, type_info: &TypeInfo) -> Self {
        Self {
            id,
            name: type_info.type_name(),
            storage_type: StorageType::default(),
            type_id: type_info.type_id(),
            drop: type_info.drop(),
            layout: type_info.layout(),
        }
    }
}

unsafe fn drop_ptr<T>(x: *mut u8) {
    x.cast::<T>().drop_in_place()
}

pub enum ComponentsError {
    ComponentAlreadyExists,
}

#[derive(Debug, Default)]
pub struct Components {
    components: Vec<ComponentInfo>,
    indices: HashMap<TypeId, usize>,
}

impl Components {
    pub(crate) fn add<T: Component>(&mut self) -> Result<ComponentId, ComponentsError> {
        let index = self.components.len();
        let info = TypeInfo::of::<T>();        
        let index_entry = self.indices.entry(info.type_id());
        if let Entry::Occupied(_) = index_entry {
            return Err(ComponentsError::ComponentAlreadyExists);
        }
        self.indices.insert(info.type_id(), index);

        let id = ComponentId::new(index);
        self.components.push(ComponentInfo::new(id, &info));

        Ok(id)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.components.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.components.len() == 0
    }

    #[inline]
    pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        self.components.get(id.index())
    }

    #[inline]
    pub fn get_id<T: Component>(&self) -> Option<ComponentId> {
        let type_id = TypeId::of::<T>();
        self.indices.get(&type_id).map(|index| ComponentId(*index))
    }

}
