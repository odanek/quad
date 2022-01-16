use std::{borrow::Cow, collections::HashMap};

use naga::{back::wgsl::WriterFlags, valid::ModuleInfo, Module};
use thiserror::Error;
use uuid::{uuid, Uuid};
use wgpu::{ShaderModuleDescriptor, ShaderSource};

use crate::{
    asset::{AssetLoader, Handle, LoadContext, LoadedAsset},
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
    source: Source,
}

impl TypeUuid for Shader {
    const TYPE_UUID: Uuid = uuid!("d95bc916-6c55-4de3-9622-37e7b6969fda");
}

impl Shader {
    pub fn from_wgsl(source: impl Into<Cow<'static, str>>) -> Shader {
        Shader {
            source: Source(source.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source(Cow<'static, str>);

#[derive(PartialEq, Eq, Debug)]
pub struct ProcessedShader(Cow<'static, str>);

impl ProcessedShader {
    pub fn get_source(&self) -> &str {
        &self.0
    }

    pub fn get_module_descriptor(&self) -> Result<ShaderModuleDescriptor, AsModuleDescriptorError> {
        Ok(ShaderModuleDescriptor {
            label: None,
            source: { ShaderSource::Wgsl(self.0.clone()) },
        })
    }
}

#[derive(Error, Debug)]
pub enum AsModuleDescriptorError {
    #[error(transparent)]
    WgslConversion(#[from] naga::back::wgsl::Error),
    #[error(transparent)]
    SpirVConversion(#[from] naga::back::spv::Error),
}

pub struct ShaderReflection {
    pub module: Module,
    pub module_info: ModuleInfo,
}

impl ShaderReflection {
    pub fn get_wgsl(&self) -> Result<String, naga::back::wgsl::Error> {
        naga::back::wgsl::write_string(&self.module, &self.module_info, WriterFlags::EXPLICIT_TYPES)
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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProcessShaderError {
    #[error("This Shader's formatdoes not support imports.")]
    ShaderFormatDoesNotSupportImports,
}

struct ShaderProcessor {}

impl ShaderProcessor {
    pub fn process(
        &self,
        shader: &Shader,
        shader_defs: &[String],
        shaders: &HashMap<Handle<Shader>, Shader>,
    ) -> Result<ProcessedShader, ProcessShaderError> {
        Ok(ProcessedShader(shader.source.0.clone()))
    }
}
