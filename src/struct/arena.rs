// Generational index, GenerationalIndexAllocator
pub struct Arena<T> {
    items: Vec<Entry<T>>,
    generation: u64,
    free_list_head: Option<usize>,
    len: usize,
}

#[derive(Clone, Debug)]
enum Entry<T> {
    Free { next_free: Option<usize> },
    Occupied { generation: u64, value: T },
}

// Combine into one u64
pub struct Index {
    index: usize,
    generation: u64,
}
