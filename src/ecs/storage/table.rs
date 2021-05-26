use std::{
    collections::HashMap,
    iter::FromIterator,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use crate::ecs::{
    component::{ComponentId, ComponentInfo, Components},
    Entity,
};

use super::BlobVec;

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

#[derive(Hash, PartialEq, Eq)]
pub struct TableIdentity {
    components: Vec<ComponentId>,
}

impl TableIdentity {
    pub fn new(components: &[ComponentId]) -> Self {
        Self {
            components: Vec::from_iter(components.iter().cloned()),
        }
    }
}

pub struct Table {
    id: TableId,
    columns: HashMap<ComponentId, Column>,
    len: usize,
    grow_amount: usize,
    capacity: usize,
}

impl Table {
    pub fn new(
        id: TableId,
        infos: &[&ComponentInfo],
        capacity: usize,
        grow_amount: usize,
    ) -> Table {
        let columns = HashMap::with_capacity(infos.len());
        for info in infos {
            columns.insert(info.id(), Column::new(info, capacity));
        }

        Self {
            id,
            columns,
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
        for column in self.columns.values_mut() {
            column.data.set_len(self.len);
        }
    }
}

pub struct Tables {
    tables: Vec<Table>,
    table_ids: HashMap<TableIdentity, TableId>,
}

impl Default for Tables {
    fn default() -> Self {
        let empty_id = TableId::empty();
        let empty_identity = TableIdentity::new(&[]);
        let empty_table = Table::new(empty_id, &[], 0, 64);

        let tables = vec![empty_table];
        let table_ids = HashMap::new();
        table_ids.insert(empty_identity, empty_id);

        Tables { tables, table_ids }
    }
}

impl Tables {
    pub fn get_id_or_insert(
        &mut self,
        component_ids: &[ComponentId],
        components: &Components,
    ) -> TableId {
        let tables = &mut self.tables;
        let identity = TableIdentity::new(component_ids);

        *self.table_ids.entry(identity).or_insert_with(move || {
            let id = TableId(tables.len());
            let infos = component_ids
                .iter()
                .map(|id| components.get_info(*id).unwrap())
                .collect::<Vec<_>>();

            tables.push(Table::new(id, &infos, 0, 64));
            id
        })
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
