use std::borrow::Cow;

use uuid::{uuid, Uuid};
use wgpu::{ShaderModuleDescriptor, ShaderSource};

use crate::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    ty::BoxedFuture,
};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ShaderId(Uuid);

impl ShaderId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ShaderId(Uuid::new_v4())
    }
}
#[derive(Debug, Clone)]
pub struct Shader {
    source: Cow<'static, str>,
}

impl TypeUuid for Shader {
    const TYPE_UUID: Uuid = uuid!("d95bc916-6c55-4de3-9622-37e7b6969fda");
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
