use std::{collections::HashSet, marker::PhantomData};

pub trait AccessIndex: Copy {
    fn index(&self) -> usize;
}

#[derive(Clone, Eq, PartialEq, Debug)]
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

    pub fn is_compatible(&self, other: &Access<T>) -> bool {
        !(intersects(&self.reads, &other.writes)
            || intersects(&other.reads, &self.writes)
            || intersects(&self.writes, &other.writes))
    }

    pub fn extend(&mut self, other: &Access<T>) {
        self.reads.extend(other.reads.iter());
        self.writes.extend(other.writes.iter());
    }

    pub fn clear(&mut self) {
        self.reads.clear();
        self.writes.clear();
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FilteredAccess<T: AccessIndex> {
    access: Access<T>,
    with: HashSet<usize>,
    without: HashSet<usize>,
}

impl<T: AccessIndex> Default for FilteredAccess<T> {
    fn default() -> Self {
        Self {
            access: Access::default(),
            with: Default::default(),
            without: Default::default(),
        }
    }
}

impl<T: AccessIndex> FilteredAccess<T> {
    #[inline]
    pub fn access(&self) -> &Access<T> {
        &self.access
    }

    pub fn has_read(&self, index: T) -> bool {
        self.access.has_read(index)
    }

    pub fn has_write(&self, index: T) -> bool {
        self.access.has_write(index)
    }

    pub fn add_read(&mut self, index: T) {
        self.access.add_read(index);
        self.add_with(index);
    }

    pub fn add_write(&mut self, index: T) {
        self.access.add_write(index);
        self.add_with(index);
    }

    pub fn add_with(&mut self, index: T) {
        self.with.insert(index.index());
    }

    pub fn add_without(&mut self, index: T) {
        self.without.insert(index.index());
    }

    pub fn is_compatible(&self, other: &FilteredAccess<T>) -> bool {
        if self.access.is_compatible(&other.access) {
            true
        } else {
            intersects(&self.with, &other.without) || intersects(&self.without, &other.with)
        }
    }

    pub fn extend(&mut self, access: &FilteredAccess<T>) {
        self.access.extend(&access.access);
        self.with.extend(access.with.iter());
        self.without.extend(access.without.iter());
    }

    pub fn extend_access(&mut self, other: &FilteredAccess<T>) {
        self.access.extend(&other.access);
    }

    pub fn extend_intersect_filter(&mut self, other: &FilteredAccess<T>) {
        self.without = self.without.intersection(&other.without).copied().collect();
        self.with = self.with.intersection(&other.with).copied().collect();
    }
}

fn intersects(left: &HashSet<usize>, right: &HashSet<usize>) -> bool {
    left.intersection(right).next().is_some()
}

#[derive(Clone, Debug)]
pub struct FilteredAccessSet<T: AccessIndex> {
    combined_access: Access<T>,
    filtered_accesses: Vec<FilteredAccess<T>>,
}

impl<T: AccessIndex> FilteredAccessSet<T> {
    #[inline]
    pub fn combined_access(&self) -> &Access<T> {
        &self.combined_access
    }

    pub fn is_compatible(&self, filtered_access: &FilteredAccess<T>) -> bool {
        if !filtered_access.access.is_compatible(&self.combined_access) {
            for current_filtered_access in &self.filtered_accesses {
                if !current_filtered_access.is_compatible(filtered_access) {
                    return false;
                }
            }
        }
        true
    }

    pub fn add(&mut self, filtered_access: FilteredAccess<T>) {
        self.combined_access.extend(&filtered_access.access);
        self.filtered_accesses.push(filtered_access);
    }

    pub fn extend(&mut self, filtered_access_set: FilteredAccessSet<T>) {
        self.combined_access
            .extend(&filtered_access_set.combined_access);
        self.filtered_accesses
            .extend(filtered_access_set.filtered_accesses);
    }

    pub fn clear(&mut self) {
        self.combined_access.clear();
        self.filtered_accesses.clear();
    }
}

impl<T: AccessIndex> Default for FilteredAccessSet<T> {
    fn default() -> Self {
        Self {
            combined_access: Default::default(),
            filtered_accesses: Vec::new(),
        }
    }
}
