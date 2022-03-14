use std::borrow::Cow;

pub use wgpu::{Backends, Features as WgpuFeatures, Limits as WgpuLimits, PowerPreference};

use crate::ecs::Resource;

#[derive(Clone, Resource)]
pub struct WgpuSettings {
    pub device_label: Option<Cow<'static, str>>,
    pub backends: Backends,
    pub power_preference: PowerPreference,
    pub features: WgpuFeatures,
    pub disabled_features: Option<WgpuFeatures>,
    pub limits: WgpuLimits,
    pub constrained_limits: Option<WgpuLimits>,
}

impl Default for WgpuSettings {
    fn default() -> Self {
        Self {
            device_label: Default::default(),
            backends: Backends::PRIMARY,
            power_preference: PowerPreference::HighPerformance,
            features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            disabled_features: None,
            limits: wgpu::Limits::default(),
            constrained_limits: None,
        }
    }
}
