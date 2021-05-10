use std::{
    alloc::{handle_alloc_error, Layout},
    cell::UnsafeCell,
    ptr::NonNull,
};

#[derive(Debug)]
pub struct BlobVec {
    item_layout: Layout,
    capacity: usize,
    len: usize,
    data: UnsafeCell<NonNull<u8>>,
    swap_scratch: UnsafeCell<NonNull<u8>>,
    drop: unsafe fn(*mut u8),
}

impl BlobVec {
    pub fn new(item_layout: Layout, drop: unsafe fn(*mut u8), capacity: usize) -> BlobVec {
        if item_layout.size() == 0 {
            BlobVec {
                swap_scratch: UnsafeCell::new(NonNull::dangling()),
                data: UnsafeCell::new(NonNull::dangling()),
                capacity: usize::MAX,
                len: 0,
                item_layout,
                drop,
            }
        } else {
            let swap_scratch = NonNull::new(unsafe { std::alloc::alloc(item_layout) })
                .unwrap_or_else(|| handle_alloc_error(item_layout));
            let mut blob_vec = BlobVec {
                swap_scratch: UnsafeCell::new(swap_scratch),
                data: UnsafeCell::new(NonNull::dangling()),
                capacity: 0,
                len: 0,
                item_layout,
                drop,
            };
            blob_vec.reserve(capacity);
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

    pub fn reserve(&mut self, amount: usize) {
        let available_space = self.capacity - self.len;
        if available_space < amount {
            self.grow(amount - available_space);
        }
    }

    fn grow(&mut self, increment: usize) {
        assert!(self.item_layout.size() != 0);

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

            self.data = UnsafeCell::new(
                NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout)),
            );
        }
        self.capacity = new_capacity;
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.capacity());
        self.len = len;
    }

    #[inline]
    pub unsafe fn get_ptr(&self) -> NonNull<u8> {
        *self.data.get()
    }

    pub fn clear(&mut self) {
        let len = self.len;
        // We set len to 0 _before_ dropping elements for unwind safety. This ensures we don't
        // accidentally drop elements twice in the event of a drop impl panicking.
        self.len = 0;
        for i in 0..len {
            unsafe {
                // NOTE: this doesn't use self.get_unchecked(i) because the debug_assert on index
                // will panic here due to self.len being set to 0
                let ptr = self.get_ptr().as_ptr().add(i * self.item_layout.size());
                (self.drop)(ptr);
            }
        }
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
                std::alloc::dealloc((*self.swap_scratch.get()).as_ptr(), self.item_layout);
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
