use super::fetch::Fetch;

pub trait FilterFetch: for<'a> Fetch<'a> {
    unsafe fn archetype_filter_fetch(&mut self, archetype_index: usize) -> bool;
    unsafe fn table_filter_fetch(&mut self, table_row: usize) -> bool;
}
