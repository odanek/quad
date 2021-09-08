use std::{
    cell::UnsafeCell,
    collections::HashMap,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use crate::ecs::{
    component::{ComponentId, ComponentInfo, ComponentTicks, Components, Tick},
    Entity,
};

use super::BlobVec;

pub struct Column {
    pub(crate) component_id: ComponentId,
    pub(crate) data: BlobVec,
    pub(crate) ticks: Vec<UnsafeCell<ComponentTicks>>, // TODO: Is UnsafeCelll needed?
}

impl Column {
    #[inline]
    pub fn new(component_info: &ComponentInfo, capacity: usize) -> Self {
        Column {
            component_id: component_info.id(),
            data: BlobVec::new(component_info.layout(), component_info.drop(), capacity),
            ticks: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub unsafe fn initialize(&mut self, row: usize, data: *mut u8, ticks: ComponentTicks) {
        self.data.initialize_unchecked(row, data);
        *self.ticks.get_unchecked_mut(row).get_mut() = ticks;
    }

    #[inline]
    pub unsafe fn replace(&mut self, row: usize, data: *mut u8, change_tick: Tick) {
        self.data.replace_unchecked(row, data);
        self.ticks
            .get_unchecked_mut(row)
            .get_mut()
            .set_changed(change_tick);
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
    pub unsafe fn swap_remove_unchecked(&mut self, row: usize) {
        self.data.swap_remove_and_drop_unchecked(row);
        self.ticks.swap_remove(row);
    }

    #[inline]
    pub unsafe fn swap_remove_and_forget_unchecked(
        &mut self,
        row: usize,
    ) -> (*mut u8, ComponentTicks) {
        let data = self.data.swap_remove_and_forget_unchecked(row);
        let ticks = self.ticks.swap_remove(row).into_inner();
        (data, ticks)
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
        self.ticks.reserve_exact(additional);
    }

    #[inline]
    pub unsafe fn get_data_ptr(&self) -> NonNull<u8> {
        self.data.get_ptr()
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, row: usize) -> *mut u8 {
        self.data.get_unchecked(row)
    }

    #[inline]
    pub unsafe fn get_ticks_mut_ptr_unchecked(&self, row: usize) -> *mut ComponentTicks {
        self.ticks.get_unchecked(row).get()
    }

    #[inline]
    pub unsafe fn clear(&mut self) {
        self.data.clear();
        self.ticks.clear();
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
            components: components.to_vec(),
        }
    }
}

pub struct Table {
    columns: HashMap<ComponentId, Column>,
    entities: Vec<Entity>, // TODO: Not needed
}

impl Table {
    pub fn new(infos: &[&ComponentInfo], capacity: usize, column_capacity: usize) -> Table {
        let mut columns = HashMap::with_capacity(infos.len());
        for info in infos {
            columns.insert(info.id(), Column::new(info, column_capacity));
        }

        Self {
            columns,
            entities: Vec::with_capacity(capacity),
        }
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

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn reserve(&mut self, additional: usize) {
        if self.entities.capacity() - self.entities.len() < additional {
            self.entities.reserve(additional);

            let new_capacity = self.entities.capacity();

            for column in self.columns.values_mut() {
                column.reserve_exact(new_capacity - column.len());
            }
        }
    }

    pub unsafe fn allocate(&mut self, entity: Entity) -> usize {
        self.reserve(1);
        let index = self.entities.len();
        self.entities.push(entity);
        for column in self.columns.values_mut() {
            column.data.set_len(index + 1);
            column
                .ticks
                .push(UnsafeCell::new(ComponentTicks::new(Default::default())));
        }
        index
    }

    pub unsafe fn swap_remove_unchecked(&mut self, row: usize) {
        self.entities.swap_remove(row);
        for column in self.columns.values_mut() {
            column.swap_remove_unchecked(row);
        }
    }

    pub unsafe fn move_to_and_forget_missing_unchecked(
        &mut self,
        row: usize,
        new_table: &mut Table,
    ) {
        let entity = self.entities.swap_remove(row);
        let new_row = new_table.allocate(entity);
        for column in self.columns.values_mut() {
            let (data, ticks) = column.swap_remove_and_forget_unchecked(row);
            if let Some(new_column) = new_table.get_column_mut(column.component_id) {
                new_column.initialize(new_row, data, ticks);
            }
        }
    }

    pub unsafe fn move_to_and_drop_missing_unchecked(&mut self, row: usize, new_table: &mut Table) {
        let entity = self.entities.swap_remove(row);
        let new_row = new_table.allocate(entity);
        for column in self.columns.values_mut() {
            if let Some(new_column) = new_table.get_column_mut(column.component_id) {
                let (data, ticks) = column.swap_remove_and_forget_unchecked(row);
                new_column.initialize(new_row, data, ticks);
            } else {
                column.swap_remove_unchecked(row);
            }
        }
    }

    pub unsafe fn move_to_superset_unchecked(&mut self, row: usize, new_table: &mut Table) {
        let entity = self.entities.swap_remove(row);
        let new_row = new_table.allocate(entity);
        for column in self.columns.values_mut() {
            let new_column = new_table.get_column_mut(column.component_id).unwrap();
            let (data, ticks) = column.swap_remove_and_forget_unchecked(row);
            new_column.initialize(new_row, data, ticks);
        }
    }

    pub unsafe fn clear(&mut self) {
        for column in self.columns.values_mut() {
            column.clear();
        }
        self.entities.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
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
        let empty_table = Table::new(&[], 0, 64);

        let tables = vec![empty_table];
        let mut table_ids = HashMap::new();
        table_ids.insert(empty_identity, empty_id);

        Tables { tables, table_ids }
    }
}

impl Tables {
    pub unsafe fn get_id_or_insert(
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
                .map(|id| components.get_info_unchecked(*id))
                .collect::<Vec<_>>();

            tables.push(Table::new(&infos, 0, 64));
            id
        })
    }

    #[inline]
    pub fn get_2_mut(&mut self, a: TableId, b: TableId) -> (&mut Table, &mut Table) {
        if a.index() > b.index() {
            let (b_slice, a_slice) = self.tables.split_at_mut(a.index());
            (&mut a_slice[0], &mut b_slice[b.index()])
        } else {
            let (a_slice, b_slice) = self.tables.split_at_mut(b.index());
            (&mut a_slice[a.index()], &mut b_slice[0])
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
