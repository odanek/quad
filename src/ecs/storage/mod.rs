mod blob_vec;
mod table;

pub use blob_vec::*;
pub use table::*;

#[derive(Default)]
pub struct Storages {    
    pub tables: Tables,
}