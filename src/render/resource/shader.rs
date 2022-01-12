use std::collections::HashMap;

use naga::{back::wgsl::WriterFlags, valid::ModuleInfo, Module};
use thiserror::Error;
use uuid::Uuid;
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

#[derive(Error, Debug)]
pub enum ShaderReflectError {
    #[error(transparent)]
    WgslParse(#[from] naga::front::wgsl::ParseError),
    #[error(transparent)]
    Validation(#[from] naga::WithSpan<naga::valid::ValidationError>),
}
#[derive(Debug, Clone)]
pub struct Shader {
    source: Source,
}

impl TypeUuid for Shader {
    const TYPE_UUID: Uuid = Uuid::parse_str("d95bc916-6c55-4de3-9622-37e7b6969fda").unwrap();
}

impl Shader {
    pub fn from_wgsl(source: String) -> Shader {
        Shader {
            source: Source(source),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source(String);

// TODO: Remove
#[derive(PartialEq, Eq, Debug)]
pub struct ProcessedShader(String);

impl ProcessedShader {
    pub fn get_source(&self) -> &str {
        &self.0
    }

    pub fn reflect(&self) -> Result<ShaderReflection, ShaderReflectError> {
        let module = naga::front::wgsl::parse_str(&self.0)?;
        let module_info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::default(),
            naga::valid::Capabilities::default(),
        )
        .validate(&module)?;

        Ok(ShaderReflection {
            module,
            module_info,
        })
    }

    pub fn get_module_descriptor(&self) -> Result<ShaderModuleDescriptor, AsModuleDescriptorError> {
        Ok(ShaderModuleDescriptor {
            label: None,
            source: {
                #[cfg(debug_assertions)]
                // This isn't neccessary, but catches errors early during hot reloading of invalid wgsl shaders.
                // Eventually, wgpu will have features that will make this unneccessary like compilation info
                // or error scopes, but until then parsing the shader twice during development the easiest solution.
                let _ = self.reflect()?;
                ShaderSource::Wgsl(self.0.into())
            },
        })
    }
}

#[derive(Error, Debug)]
pub enum AsModuleDescriptorError {
    #[error(transparent)]
    ShaderReflectError(#[from] ShaderReflectError),
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

            let mut shader = match ext {
                "wgsl" => Shader::from_wgsl(String::from_utf8(Vec::from(bytes))?),
                _ => panic!("Unhandled shader extension: {}", ext),
            };

            let mut asset = LoadedAsset::new(shader);

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
