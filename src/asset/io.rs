mod file_asset_io;

use std::{
    io,
    path::{Path, PathBuf},
};

use downcast_rs::Downcast;
use thiserror::Error;

use crate::ty::BoxedFuture;

#[derive(Error, Debug)]
pub enum AssetIoError {
    #[error("path not found: {0}")]
    NotFound(PathBuf),
    #[error("encountered an io error while loading asset: {0}")]
    Io(#[from] io::Error),
    // #[error("failed to watch path: {0}")]
    // PathWatchError(PathBuf),
}

pub trait AssetIo: Downcast + Send + Sync + 'static {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>>;
    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError>;
    fn is_directory(&self, path: &Path) -> bool;
    fn watch_path_for_changes(&self, path: &Path) -> Result<(), AssetIoError>;
    fn watch_for_changes(&self) -> Result<(), AssetIoError>;
}
