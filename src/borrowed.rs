use {
    crate::{polyfill::layout_fits_in, Memory, Storage},
    core::{
        alloc::{AllocError, Layout},
        mem::MaybeUninit,
        ptr,
    },
};

/// A single storage which stores memory in borrowed space.
///
/// The `DataStore` type parameter determines the layout of the inline storage.
/// (It would be nice to use `const LAYOUT: Layout` instead, but the needed
/// features are currently a little *too* incomplete to be usable here yet.)
pub struct BorrowedStorage<'a, DataStore> {
    data: &'a mut MaybeUninit<DataStore>,
}

impl<'a, DataStore> BorrowedStorage<'a, DataStore> {
    pub fn new(store: &'a mut MaybeUninit<DataStore>) -> Self {
        Self { data: store }
    }

    pub fn fits(&self, needed_layout: Layout) -> bool {
        let available_layout = Layout::new::<DataStore>();
        layout_fits_in(needed_layout, available_layout)
    }
}

unsafe impl<DataStore> Storage for BorrowedStorage<'_, DataStore> {
    type Handle = ();

    fn allocate(&mut self, layout: Layout) -> Result<Self::Handle, AllocError> {
        if self.fits(layout) {
            Ok(())
        } else {
            Err(AllocError)
        }
    }

    unsafe fn deallocate(&mut self, _handle: Self::Handle, _layout: Layout) {}

    unsafe fn resolve(&self, _handle: Self::Handle, layout: Layout) -> &Memory {
        &*ptr::from_raw_parts(self.data.as_ptr().cast(), layout.size())
    }

    unsafe fn resolve_mut(&mut self, _handle: Self::Handle, layout: Layout) -> &mut Memory {
        &mut *ptr::from_raw_parts_mut(self.data.as_mut_ptr().cast(), layout.size())
    }

    unsafe fn grow(
        &mut self,
        handle: Self::Handle,
        _old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError> {
        if self.fits(new_layout) {
            Ok(handle)
        } else {
            Err(AllocError)
        }
    }

    unsafe fn shrink(
        &mut self,
        handle: Self::Handle,
        _old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError> {
        debug_assert!(self.fits(new_layout));
        Ok(handle)
    }
}
