use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static TEXTURE_ID: AtomicU64 = AtomicU64::new(0);
static TEXTURE_VIEW_ID: AtomicU64 = AtomicU64::new(0);
static SAMPLER_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct TextureId(u64);

#[derive(Clone, Debug)]
pub struct Texture {
    id: TextureId,
    value: Arc<wgpu::Texture>,
}

impl Texture {
    #[inline]
    pub fn id(&self) -> TextureId {
        self.id
    }

    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> TextureView {
        TextureView::from(self.value.create_view(desc))
    }
}

impl From<wgpu::Texture> for Texture {
    fn from(value: wgpu::Texture) -> Self {
        Texture {
            id: TextureId(TEXTURE_ID.fetch_add(1, Ordering::Relaxed)),
            value: Arc::new(value),
        }
    }
}

impl Deref for Texture {
    type Target = wgpu::Texture;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct TextureViewId(u64);

#[derive(Clone, Debug)]
pub enum TextureViewValue {
    TextureView(Arc<wgpu::TextureView>),
    SurfaceTexture {
        view: Arc<wgpu::TextureView>,
        texture: Arc<wgpu::SurfaceTexture>,
    },
}

#[derive(Clone, Debug)]
pub struct TextureView {
    id: TextureViewId,
    value: TextureViewValue,
}

impl TextureView {
    #[inline]
    pub fn id(&self) -> TextureViewId {
        self.id
    }

    #[inline]
    pub fn take_surface_texture(self) -> Option<wgpu::SurfaceTexture> {
        match self.value {
            TextureViewValue::TextureView(_) => None,
            TextureViewValue::SurfaceTexture { texture, .. } => Arc::try_unwrap(texture).ok(),
        }
    }
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        TextureView {
            id: TextureViewId(TEXTURE_VIEW_ID.fetch_add(1, Ordering::Relaxed)),
            value: TextureViewValue::TextureView(Arc::new(value)),
        }
    }
}

impl From<wgpu::SurfaceTexture> for TextureView {
    fn from(value: wgpu::SurfaceTexture) -> Self {
        let texture = Arc::new(value);
        let view = Arc::new(texture.texture.create_view(&Default::default()));

        TextureView {
            id: TextureViewId(TEXTURE_VIEW_ID.fetch_add(1, Ordering::Relaxed)),
            value: TextureViewValue::SurfaceTexture { texture, view },
        }
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match &self.value {
            TextureViewValue::TextureView(value) => value,
            TextureViewValue::SurfaceTexture { view, .. } => view,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct SamplerId(u64);

#[derive(Clone, Debug)]
pub struct Sampler {
    id: SamplerId,
    value: Arc<wgpu::Sampler>,
}

impl Sampler {
    #[inline]
    pub fn id(&self) -> SamplerId {
        self.id
    }
}

impl From<wgpu::Sampler> for Sampler {
    fn from(value: wgpu::Sampler) -> Self {
        Sampler {
            id: SamplerId(SAMPLER_ID.fetch_add(1, Ordering::Relaxed)),
            value: Arc::new(value),
        }
    }
}

impl Deref for Sampler {
    type Target = wgpu::Sampler;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
