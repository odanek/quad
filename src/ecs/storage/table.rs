use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use crate::ecs::{Entity, component::{ComponentId, ComponentInfo}};

use super::BlobVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableId(usize);

impl TableId {
    #[inline]
    pub fn new(id: usize) -> Self {
        TableId(id)
    }

    #[inline]
    pub const fn empty() -> TableId {
        TableId(0)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

pub struct Column {
    pub(crate) component: ComponentId,
    pub(crate) data: BlobVec,
}

impl Column {
    #[inline]
    pub fn new(component_info: &ComponentInfo, capacity: usize) -> Self {
        Column {
            component: component_info.id(),
            data: BlobVec::new(component_info.layout(), component_info.drop(), capacity),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    #[inline]
    pub unsafe fn get_ptr(&self) -> NonNull<u8> {
        self.data.get_ptr()
    }

    #[inline]
    pub unsafe fn set_unchecked(&self, row: usize, data: *mut u8) {
        self.data.set_unchecked(row, data);
    }
}

pub struct Table {
    columns: HashMap<ComponentId, Column>,
    len: usize,
    grow_amount: usize,
    capacity: usize,
}

impl Table {
    pub fn new(infos: &[ComponentInfo], capacity: usize, grow_amount: usize) -> Table {
        let columns = HashMap::with_capacity(infos.len());
        for info in infos {
            columns.insert(info.id(), Column::new(info, capacity));
        }

        Self {
            columns,
            // entities: Vec::with_capacity(capacity),
            // archetypes: Vec::new(),
            len: 0,
            grow_amount,
            capacity,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn get_column(&self, component_id: ComponentId) -> Option<&Column> {
        self.columns.get(&component_id)
    }

    #[inline]
    pub fn get_column_mut(&mut self, component_id: ComponentId) -> Option<&mut Column> {
        self.columns.get_mut(&component_id)
    }

    #[inline]
    pub fn has_column(&self, component_id: ComponentId) -> bool {
        self.columns.contains_key(&component_id)
    }

    pub fn reserve(&mut self, amount: usize) {
        let available_space = self.capacity - self.len();
        if available_space < amount {
            let min_capacity = self.len() + amount;
            let new_capacity =
                ((min_capacity + self.grow_amount - 1) / self.grow_amount) * self.grow_amount;
            let reserve_amount = new_capacity - self.len();
            for column in self.columns.values_mut() {
                column.reserve(reserve_amount);
            }
            // self.entities.reserve(reserve_amount);
            self.capacity = new_capacity;
        }
    }

    pub unsafe fn allocate(&mut self, entity: Entity) {
        self.reserve(1);
        self.len += 1;
        // self.entities.push(entity);
        for column in self.columns.values_mut() {
            column.data.set_len(self.len);
        }
    }
}

pub struct Tables {
    tables: Vec<Table>,
}

impl Default for Tables {
    fn default() -> Self {
        let empty_table = Table::with_capacity(0, 0, 64);
        Tables {
            tables: vec![empty_table],
        }
    }

    
}

impl Index<TableId> for Tables {
    type Output = Table;

    #[inline]
    fn index(&self, index: TableId) -> &Self::Output {
        &self.tables[index.index()]
    }
}

impl IndexMut<TableId> for Tables {
    #[inline]
    fn index_mut(&mut self, index: TableId) -> &mut Self::Output {
        &mut self.tables[index.index()]
    }
}
