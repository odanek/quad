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
pub use loader::{
    update_asset_storage_system, Asset, AssetDynamic, AssetLoader, LoadContext, LoadedAsset,
};

use crate::{
    app::{App, MainStage},
    ecs::Resource,
    tasks::IoTaskPool,
};

#[derive(Clone, Resource)]
pub struct AssetServerSettings {
    pub asset_folder: String,
}

impl Default for AssetServerSettings {
    fn default() -> Self {
        Self {
            asset_folder: "assets".to_string(),
        }
    }
}

pub fn asset_plugin(app: &mut App) {
    let task_pool = app.resource::<IoTaskPool>().0.clone();
    let settings = app.resource::<AssetServerSettings>();
    let source = Box::new(FileAssetIo::new(&settings.asset_folder));
    let asset_server = AssetServer::with_boxed_io(source, task_pool);
    app.insert_resource(asset_server);
    app.add_system_to_stage(MainStage::PreUpdate, &free_unused_assets_system);
}
