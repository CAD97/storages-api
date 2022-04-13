use core::{
    alloc::Layout,
    ptr::{self, Pointee},
};

// For now, we're just assuming that metadata passed in describes a valid
// layout. Ideally, we shouldn't have to, but there's no current way to go from
// metadata to size without breaking the "must be <= isize::MAX" rule.
pub(crate) unsafe fn layout_for_metadata<T: ?Sized>(
    meta: <T as Pointee>::Metadata,
) -> Option<Layout> {
    let ptr: *const T = ptr::from_raw_parts(ptr::null(), meta);

    // We *need* a way to check that this is sound
    unsafe {
        // SAFETY: it's *not*, but there's no way to pre-check
        Some(Layout::for_value_raw(ptr))
    }
}

pub(crate) fn layout_fits_in(inner: Layout, outer: Layout) -> bool {
    inner.align() <= outer.align() && inner.size() <= outer.size()
}
