use anyhow::Result;
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use crate::ty::BoxedFuture;

use super::{AssetIo, AssetIoError};

pub struct FileAssetIo {
    root_path: PathBuf,
}

impl FileAssetIo {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileAssetIo {
            root_path: Self::get_root_path().join(path.as_ref()),
        }
    }

    pub fn get_root_path() -> PathBuf {
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(manifest_dir)
        } else {
            env::current_exe()
                .map(|path| {
                    path.parent()
                        .map(|exe_parent_path| exe_parent_path.to_owned())
                        .unwrap()
                })
                .unwrap()
        }
    }
}

impl AssetIo for FileAssetIo {
    fn load_path<'a>(&'a self, path: &'a Path) -> BoxedFuture<'a, Result<Vec<u8>, AssetIoError>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            let full_path = self.root_path.join(path);
            match File::open(&full_path) {
                Ok(mut file) => {
                    file.read_to_end(&mut bytes)?;
                }
                Err(e) => {
                    return if e.kind() == std::io::ErrorKind::NotFound {
                        Err(AssetIoError::NotFound(full_path))
                    } else {
                        Err(e.into())
                    };
                }
            }
            Ok(bytes)
        })
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        let root_path = self.root_path.to_owned();
        Ok(Box::new(fs::read_dir(root_path.join(path))?.map(
            move |entry| {
                let path = entry.unwrap().path();
                path.strip_prefix(&root_path).unwrap().to_owned()
            },
        )))
    }

    fn watch_path_for_changes(&self, _path: &Path) -> Result<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> Result<(), AssetIoError> {
        Ok(())
    }

    fn is_directory(&self, path: &Path) -> bool {
        self.root_path.join(path).is_dir()
    }
}
