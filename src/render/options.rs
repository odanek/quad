pub use wgpu::{Backends, Features, Limits, PowerPreference};

use crate::ecs::Resource;

#[derive(Clone, Resource)]
pub struct WgpuOptions {
    pub device_label: Option<String>,
    pub backends: Backends,
    pub power_preference: PowerPreference,
    pub features: Features,
    pub limits: Limits,
}

impl Default for WgpuOptions {
    fn default() -> Self {
        Self {
            device_label: Default::default(),
            backends: Backends::PRIMARY,
            power_preference: PowerPreference::HighPerformance,
            features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            limits: Limits::default(),
        }
    }
}
