use std::{
    ops::{Bound, Deref, RangeBounds},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

static BUFFER_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct BufferId(u64);

#[derive(Clone, Debug)]
pub struct Buffer {
    id: BufferId,
    value: Arc<wgpu::Buffer>,
}

impl Buffer {
    #[inline]
    pub fn id(&self) -> BufferId {
        self.id
    }

    pub fn slice(&self, bounds: impl RangeBounds<wgpu::BufferAddress>) -> BufferSlice {
        BufferSlice {
            id: self.id,
            // need to compute and store this manually because wgpu doesn't export offset on wgpu::BufferSlice
            offset: match bounds.start_bound() {
                Bound::Included(&bound) => bound,
                Bound::Excluded(&bound) => bound + 1,
                Bound::Unbounded => 0,
            },
            value: self.value.slice(bounds),
        }
    }

    #[inline]
    pub fn unmap(&self) {
        self.value.unmap()
    }
}

impl From<wgpu::Buffer> for Buffer {
    fn from(value: wgpu::Buffer) -> Self {
        Buffer {
            id: BufferId(BUFFER_ID.fetch_add(1, Ordering::Relaxed)),
            value: Arc::new(value),
        }
    }
}

impl Deref for Buffer {
    type Target = wgpu::Buffer;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub struct BufferSlice<'a> {
    id: BufferId,
    offset: wgpu::BufferAddress,
    value: wgpu::BufferSlice<'a>,
}

impl<'a> BufferSlice<'a> {
    #[inline]
    pub fn id(&self) -> BufferId {
        self.id
    }

    #[inline]
    pub fn offset(&self) -> wgpu::BufferAddress {
        self.offset
    }
}

impl<'a> Deref for BufferSlice<'a> {
    type Target = wgpu::BufferSlice<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
