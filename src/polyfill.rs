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

// CRIMES 😈😈😈😈😈😈
extern "Rust" {
    // This is the magic symbol to call the global alloc error handler.  rustc generates
    // it to call `__rg_oom` if there is a `#[alloc_error_handler]`, or to call the
    // default implementations below (`__rdl_oom`) otherwise.
    fn __rust_alloc_error_handler(size: usize, align: usize) -> !;
}

pub(crate) fn handle_alloc_error(layout: Layout) -> ! {
    unsafe { __rust_alloc_error_handler(layout.size(), layout.align()) }
}

pub const fn is_zst<T: Copy>() -> bool {
    core::mem::size_of::<T>() == 0
}

pub struct Bool<const B: bool>;
pub trait True {}
impl True for Bool<true> {}
