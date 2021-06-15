use std::{collections::HashSet, marker::PhantomData};

pub trait AccessIndex {
    fn index(&self) -> usize;
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Access<T: AccessIndex> {
    reads_and_writes: HashSet<usize>,
    writes: HashSet<usize>,
    marker: PhantomData<T>,
}

impl<T: AccessIndex> Default for Access<T> {
    fn default() -> Self {
        Self {
            reads_and_writes: Default::default(),
            writes: Default::default(),
            marker: PhantomData,
        }
    }
}
