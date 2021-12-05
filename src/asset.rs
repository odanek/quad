mod asset_server;
mod assets;
mod handle;
mod info;
mod io;
mod loader;
mod path;

pub use asset_server::{free_unused_assets_system, AssetServer};
pub use assets::{AssetEvent, Assets};
pub use handle::{Handle, HandleId, HandleUntyped};
pub use io::FileAssetIo;
pub use loader::{update_asset_storage_system, Asset, AssetDynamic};

use crate::ecs::Resource;

pub struct AssetServerSettings {
    pub asset_folder: String,
}

impl Resource for AssetServerSettings {}

impl Default for AssetServerSettings {
    fn default() -> Self {
        Self {
            asset_folder: "assets".to_string(),
        }
    }
}
