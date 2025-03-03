use std::{
    alloc::{Layout, handle_alloc_error},
    ptr::NonNull,
};

#[derive(Debug)]
pub struct BlobVec {
    item_layout: Layout,
    capacity: usize,
    len: usize,
    data: NonNull<u8>,
    swap_scratch: NonNull<u8>,
    drop: unsafe fn(*mut u8),
}

impl BlobVec {
    pub fn new(item_layout: Layout, drop: unsafe fn(*mut u8), capacity: usize) -> BlobVec {
        if item_layout.size() == 0 {
            BlobVec {
                swap_scratch: NonNull::dangling(),
                data: NonNull::dangling(),
                capacity: usize::MAX,
                len: 0,
                item_layout,
                drop,
            }
        } else {
            let swap_scratch = NonNull::new(unsafe { std::alloc::alloc(item_layout) })
                .unwrap_or_else(|| handle_alloc_error(item_layout));
            let mut blob_vec = BlobVec {
                swap_scratch,
                data: NonNull::dangling(),
                capacity: 0,
                len: 0,
                item_layout,
                drop,
            };
            blob_vec.reserve_exact(capacity);
            blob_vec
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let available_space = self.capacity - self.len;
        if available_space < additional {
            self.grow_exact(additional - available_space);
        }
    }

    fn grow_exact(&mut self, increment: usize) {
        debug_assert!(self.item_layout.size() != 0);

        let new_capacity = self.capacity + increment;
        let new_layout =
            array_layout(&self.item_layout, new_capacity).expect("array layout should be valid");
        unsafe {
            let new_data = if self.capacity == 0 {
                std::alloc::alloc(new_layout)
            } else {
                std::alloc::realloc(
                    self.get_ptr().as_ptr(),
                    array_layout(&self.item_layout, self.capacity)
                        .expect("array layout should be valid"),
                    new_layout.size(),
                )
            };

            self.data = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
        }
        self.capacity = new_capacity;
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity());
        self.len = len;
    }

    #[inline]
    pub unsafe fn get_ptr(&self) -> NonNull<u8> {
        self.data
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        self.get_ptr().as_ptr().add(index * self.item_layout.size())
    }

    #[inline]
    pub unsafe fn initialize_unchecked(&mut self, index: usize, value: *mut u8) {
        debug_assert!(index < self.len);
        let ptr = self.get_unchecked(index);
        std::ptr::copy_nonoverlapping(value, ptr, self.item_layout.size());
    }

    #[inline]
    pub unsafe fn replace_unchecked(&mut self, index: usize, value: *mut u8) {
        debug_assert!(index < self.len);
        let ptr = self.get_unchecked(index);
        (self.drop)(ptr);
        std::ptr::copy_nonoverlapping(value, ptr, self.item_layout.size());
    }

    #[inline]
    pub unsafe fn swap_remove_and_drop_unchecked(&mut self, index: usize) {
        debug_assert!(index < self.len);
        let value = self.swap_remove_and_forget_unchecked(index);
        (self.drop)(value)
    }

    pub fn clear(&mut self) {
        let len = self.len;
        self.len = 0;
        for i in 0..len {
            unsafe {
                let ptr = self.get_ptr().as_ptr().add(i * self.item_layout.size());
                (self.drop)(ptr);
            }
        }
    }

    #[inline]
    pub unsafe fn swap_remove_and_forget_unchecked(&mut self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len);
        let last = self.len - 1;
        let swap_scratch = self.swap_scratch.as_ptr();
        let src = self.get_unchecked(index);
        std::ptr::copy_nonoverlapping(src, swap_scratch, self.item_layout.size());
        if last != index {
            std::ptr::copy_nonoverlapping(self.get_unchecked(last), src, self.item_layout.size());
        }
        self.len -= 1;
        swap_scratch
    }
}

impl Drop for BlobVec {
    fn drop(&mut self) {
        self.clear();
        if self.item_layout.size() > 0 {
            unsafe {
                std::alloc::dealloc(
                    self.get_ptr().as_ptr(),
                    array_layout(&self.item_layout, self.capacity)
                        .expect("array layout should be valid"),
                );
                std::alloc::dealloc((self.swap_scratch).as_ptr(), self.item_layout);
            }
        }
    }
}

/// From https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html
fn array_layout(layout: &Layout, n: usize) -> Option<Layout> {
    let (array_layout, offset) = repeat_layout(layout, n)?;
    debug_assert_eq!(layout.size(), offset);
    Some(array_layout)
}

// TODO: replace with Layout::repeat if/when it stabilizes
/// From https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html
fn repeat_layout(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
    // This cannot overflow. Quoting from the invariant of Layout:
    // > `size`, when rounded up to the nearest multiple of `align`,
    // > must not overflow (i.e., the rounded value must be less than
    // > `usize::MAX`)
    let padded_size = layout.size() + padding_needed_for(layout, layout.align());
    let alloc_size = padded_size.checked_mul(n)?;

    // SAFETY: self.align is already known to be valid and alloc_size has been
    // padded already.
    unsafe {
        Some((
            Layout::from_size_align_unchecked(alloc_size, layout.align()),
            padded_size,
        ))
    }
}

/// From https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html
const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let len = layout.size();

    // Rounded up value is:
    //   len_rounded_up = (len + align - 1) & !(align - 1);
    // and then we return the padding difference: `len_rounded_up - len`.
    //
    // We use modular arithmetic throughout:
    //
    // 1. align is guaranteed to be > 0, so align - 1 is always
    //    valid.
    //
    // 2. `len + align - 1` can overflow by at most `align - 1`,
    //    so the &-mask with `!(align - 1)` will ensure that in the
    //    case of overflow, `len_rounded_up` will itself be 0.
    //    Thus the returned padding, when added to `len`, yields 0,
    //    which trivially satisfies the alignment `align`.
    //
    // (Of course, attempts to allocate blocks of memory whose
    // size and padding overflow in the above manner should cause
    // the allocator to yield an error anyway.)

    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}
