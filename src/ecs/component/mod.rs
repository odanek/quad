use std::{alloc::Layout, any::{TypeId, type_name}, collections::{HashMap, hash_map::Entry}};

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
    pub fn index(self) -> usize {
        self.0 as usize
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

impl ComponentInfo {
    #[inline]
    pub fn id(&self) -> ComponentId {
        self.id
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
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

    pub fn new<T: Component>(id: ComponentId) -> Self {
        Self {
            id,
            name: type_name::<T>().to_owned(),
            storage_type: StorageType::default(),
            type_id: TypeId::of::<T>(),
            drop: drop_ptr::<T>,
            layout: Layout::new::<T>(),
        }
    }
}

unsafe fn drop_ptr<T>(x: *mut u8) {
    x.cast::<T>().drop_in_place()
}

#[derive(Debug, Default)]
pub struct Components {
    components: Vec<ComponentInfo>,
    indices: HashMap<TypeId, usize>,
}

impl Components {
    // pub(crate) fn add<T: Component>(&mut self) -> Result<ComponentId, ComponentsError> {
    //     let index = self.components.len();
    //     let type_id = TypeId::of::<T>();
    //     let index_entry = self.indices.entry(type_id);
    //     if let Entry::Occupied(_) = index_entry {
    //         return Err(ComponentsError::ComponentAlreadyExists {
    //             type_id,
    //             name: descriptor.name,
    //         });
    //     }
    //     self.indices.insert(type_id, index);

    //     self.components
    //         .push(ComponentInfo::new(ComponentId(index), descriptor));

    //     Ok(ComponentId(index))
    // }
    
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
        self.components.get(id.0)
    }
}
