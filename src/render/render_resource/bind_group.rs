use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

static BIND_GROUP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct BindGroupId(u64);

#[derive(Clone, Debug)]
pub struct BindGroup {
    id: BindGroupId,
    value: Arc<wgpu::BindGroup>,
}

impl BindGroup {
    #[inline]
    pub fn id(&self) -> BindGroupId {
        self.id
    }
}

impl From<wgpu::BindGroup> for BindGroup {
    fn from(value: wgpu::BindGroup) -> Self {
        BindGroup {
            id: BindGroupId(BIND_GROUP_ID.fetch_add(1, Ordering::Relaxed)),
            value: Arc::new(value),
        }
    }
}

impl Deref for BindGroup {
    type Target = wgpu::BindGroup;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
