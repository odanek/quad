use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

static BIND_GROUP_LAYOUT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct BindGroupLayoutId(u64);

#[derive(Clone, Debug)]
pub struct BindGroupLayout {
    id: BindGroupLayoutId,
    value: Arc<wgpu::BindGroupLayout>,
}

impl PartialEq for BindGroupLayout {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl BindGroupLayout {
    #[inline]
    pub fn id(&self) -> BindGroupLayoutId {
        self.id
    }

    #[inline]
    pub fn value(&self) -> &wgpu::BindGroupLayout {
        &self.value
    }
}

impl From<wgpu::BindGroupLayout> for BindGroupLayout {
    fn from(value: wgpu::BindGroupLayout) -> Self {
        BindGroupLayout {
            id: BindGroupLayoutId(BIND_GROUP_LAYOUT_ID.fetch_add(1, Ordering::Relaxed)),
            value: Arc::new(value),
        }
    }
}

impl Deref for BindGroupLayout {
    type Target = wgpu::BindGroupLayout;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
