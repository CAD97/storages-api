use core::{
    alloc::Layout,
    mem,
    ptr::{self, Pointee},
};

// For now, we're just assuming that metadata passed in describes a valid
// layout. Ideally, we shouldn't have to, but there's no current way to go from
// metadata to size without breaking the "must be <= isize::MAX" rule.
pub(crate) fn layout_for_meta<T: ?Sized>(meta: <T as Pointee>::Metadata) -> Option<Layout> {
    let ptr: *const T = ptr::from_raw_parts(ptr::null(), meta);

    // We *need* a way to check that these are safe
    unsafe {
        // SAFETY: it's *not*, but there's no way to pre-check
        let size = mem::size_of_val_raw(ptr);
        // SAFETY: it's *not*, but there's no way to pre-check
        let align = mem::align_of_val_raw(ptr);

        // SAFETY: sizeof/alignof return valid size/align
        Some(Layout::from_size_align_unchecked(size, align))
    }
}

pub(crate) fn layout_fits_in(inner: Layout, outer: Layout) -> bool {
    inner.align() <= outer.align() && inner.size() <= outer.size()
}
