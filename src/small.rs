use {
    crate::{AllocStorage, InlineStorage, Memory, Storage},
    core::{
        alloc::{AllocError, Allocator, Layout},
        hint::unreachable_unchecked,
        ptr::copy_nonoverlapping,
    },
};

/// A single storage which stores memory inline if it fits, otherwise falling
/// back to using an [`Allocator`].
///
/// The `DataStore` type parameter determines the layout of the inline storage.
/// (It would be nice to use `const LAYOUT: Layout` instead, but the needed
/// features are currently a little *too* incomplete to be usable here.)
pub struct SmallStorage<DataStore, A: Allocator> {
    inline: InlineStorage<DataStore>,
    outline: AllocStorage<A>,
}

impl<DataStore, A: Allocator> SmallStorage<DataStore, A> {
    pub fn new(alloc: A) -> Self {
        Self {
            inline: InlineStorage::new(),
            outline: AllocStorage::new(alloc),
        }
    }

    const OUTLINE_HANDLE_LAYOUT: Layout = Layout::new::<<AllocStorage<A> as Storage>::Handle>();
}

unsafe impl<DataStore, A: Allocator> Storage for SmallStorage<DataStore, A> {
    type Handle = ();

    fn allocate(&mut self, layout: Layout) -> Result<Self::Handle, AllocError> {
        if self.inline.fits(layout) {
            self.inline.allocate(layout)
        } else {
            let addr = self.outline.allocate(layout)?;
            let addr_handle = self.inline.allocate(Self::OUTLINE_HANDLE_LAYOUT)?;
            unsafe {
                *self
                    .inline
                    .resolve_mut(addr_handle, Self::OUTLINE_HANDLE_LAYOUT)
                    .as_mut_ptr()
                    .cast() = addr;
            }
            Ok(addr_handle)
        }
    }

    unsafe fn deallocate(&mut self, handle: Self::Handle, layout: Layout) {
        if self.inline.fits(layout) {
            self.inline.deallocate(handle, layout)
        } else {
            let addr = *self
                .inline
                .resolve_mut(handle, Self::OUTLINE_HANDLE_LAYOUT)
                .as_ptr()
                .cast();
            self.inline.deallocate(handle, Self::OUTLINE_HANDLE_LAYOUT);
            self.outline.deallocate(addr, layout);
        }
    }

    unsafe fn resolve(&self, handle: Self::Handle, layout: Layout) -> &Memory {
        if self.inline.fits(layout) {
            self.inline.resolve(handle, layout)
        } else {
            let addr = *self
                .inline
                .resolve(handle, Self::OUTLINE_HANDLE_LAYOUT)
                .as_ptr()
                .cast();
            self.outline.resolve(addr, layout)
        }
    }

    unsafe fn resolve_mut(&mut self, handle: Self::Handle, layout: Layout) -> &mut Memory {
        if self.inline.fits(layout) {
            self.inline.resolve_mut(handle, layout)
        } else {
            let addr = *self
                .inline
                .resolve_mut(handle, Self::OUTLINE_HANDLE_LAYOUT)
                .as_mut_ptr()
                .cast();
            self.outline.resolve_mut(addr, layout)
        }
    }

    unsafe fn grow(
        &mut self,
        handle: Self::Handle,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError> {
        match (self.inline.fits(old_layout), self.inline.fits(new_layout)) {
            (true, true) => self.inline.grow(handle, old_layout, new_layout),
            (false, true) => unreachable_unchecked(),
            (false, false) => {
                let addr = self
                    .inline
                    .resolve_mut(handle, Self::OUTLINE_HANDLE_LAYOUT)
                    .as_mut_ptr()
                    .cast();
                *addr = self.outline.grow(*addr, old_layout, new_layout)?;
                Ok(handle)
            },
            (true, false) => {
                if !self.inline.fits(Self::OUTLINE_HANDLE_LAYOUT) {
                    return Err(AllocError);
                }

                let addr = self.outline.allocate(new_layout)?;
                let new_ptr = self.outline.resolve_mut(addr, new_layout);
                let old_ptr = self.inline.resolve_mut(handle, old_layout);

                copy_nonoverlapping(
                    old_ptr.as_mut_ptr(),
                    new_ptr.as_mut_ptr(),
                    old_layout.size(),
                );

                self.inline.deallocate(handle, old_layout);
                let addr_handle = self
                    .inline
                    .allocate(Self::OUTLINE_HANDLE_LAYOUT)
                    .unwrap_unchecked();
                *self
                    .inline
                    .resolve_mut(addr_handle, Self::OUTLINE_HANDLE_LAYOUT)
                    .as_mut_ptr()
                    .cast() = addr;
                Ok(addr_handle)
            },
        }
    }

    unsafe fn shrink(
        &mut self,
        handle: Self::Handle,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError> {
        match (self.inline.fits(old_layout), self.inline.fits(new_layout)) {
            (true, true) => self.inline.shrink(handle, old_layout, new_layout),
            (true, false) => unreachable_unchecked(),
            (false, false) => {
                let addr = self
                    .inline
                    .resolve_mut(handle, Self::OUTLINE_HANDLE_LAYOUT)
                    .as_mut_ptr()
                    .cast();
                *addr = self.outline.shrink(*addr, old_layout, new_layout)?;
                Ok(handle)
            },
            (false, true) => {
                todo!();
            },
        }
    }
}
