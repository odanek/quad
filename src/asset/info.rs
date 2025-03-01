use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use super::path::{AssetPath, LabelId};

#[derive(Clone, Debug)]
pub struct SourceMeta {
    pub assets: Vec<AssetMeta>,
}

#[derive(Clone, Debug)]
pub struct AssetMeta {
    pub label: Option<String>,
    pub dependencies: Vec<AssetPath<'static>>,
    pub type_uuid: TypeId,
}

#[derive(Clone, Debug)]
pub struct SourceInfo {
    pub meta: Option<SourceMeta>,
    // pub path: PathBuf,
    pub asset_types: HashMap<LabelId, TypeId>,
    pub load_state: LoadState,
    pub committed_assets: HashSet<LabelId>,
    pub version: usize,
}

impl SourceInfo {
    pub fn is_loaded(&self) -> bool {
        self.meta
            .as_ref()
            .is_some_and(|meta| self.committed_assets.len() == meta.assets.len())
    }

    pub fn get_asset_type(&self, label_id: LabelId) -> Option<TypeId> {
        self.asset_types.get(&label_id).cloned()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LoadState {
    NotLoaded,
    Loading,
    Loaded,
    Failed,
    Unloaded,
}
