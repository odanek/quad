use std::{alloc::Layout, any::{TypeId, type_name}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    type_id: TypeId,
    layout: Layout,
    drop: unsafe fn(*mut u8),
    type_name: &'static str,
}

impl TypeInfo {
    pub fn of<T: Send + Sync + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop: Self::drop_ptr::<T>,
            type_name: type_name::<T>(),
        }
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
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub(crate) unsafe fn drop_ptr<T>(x: *mut u8) {
        x.cast::<T>().drop_in_place()
    }
}
