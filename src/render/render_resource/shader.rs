use std::{
    borrow::Cow,
    sync::atomic::{AtomicU64, Ordering},
};

use wgpu::{ShaderModuleDescriptor, ShaderSource};

use crate::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    ty::BoxedFuture,
};

static SHADER_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ShaderId(u64);

impl ShaderId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ShaderId(SHADER_ID.fetch_add(1, Ordering::Relaxed))
    }
}
#[derive(Debug, Clone)]
pub struct Shader {
    source: Cow<'static, str>,
}

impl Shader {
    pub fn from_wgsl(source: impl Into<Cow<'static, str>>) -> Shader {
        Shader {
            source: source.into(),
        }
    }

    pub(crate) fn get_module_descriptor(&self) -> ShaderModuleDescriptor {
        ShaderModuleDescriptor {
            label: None,
            source: { ShaderSource::Wgsl(self.source.clone()) },
        }
    }
}

#[derive(Default)]
pub struct ShaderLoader;

impl AssetLoader for ShaderLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let ext = load_context.path().extension().unwrap().to_str().unwrap();

            let shader = match ext {
                "wgsl" => Shader::from_wgsl(String::from_utf8(Vec::from(bytes))?),
                _ => panic!("Unhandled shader extension: {}", ext),
            };

            let asset = LoadedAsset::new(shader);

            load_context.set_default_asset(asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wgsl"]
    }
}
