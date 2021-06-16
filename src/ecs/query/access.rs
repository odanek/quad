use std::{collections::HashSet, marker::PhantomData};

pub trait AccessIndex {
    fn index(&self) -> usize;
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Access<T: AccessIndex> {
    reads: HashSet<usize>,
    writes: HashSet<usize>,
    marker: PhantomData<T>,
}

impl<T: AccessIndex> Default for Access<T> {
    fn default() -> Self {
        Self {
            reads: Default::default(),
            writes: Default::default(),
            marker: PhantomData,
        }
    }
}

impl<T: AccessIndex> Access<T> {
    pub fn has_read(&self, index: T) -> bool {
        self.reads.contains(&index.index())
    }

    pub fn has_write(&self, index: T) -> bool {
        self.writes.contains(&index.index())
    }

    pub fn add_read(&mut self, index: T) {
        self.reads.insert(index.index());
    }

    pub fn add_write(&mut self, index: T) {
        self.writes.insert(index.index());
    }
}
