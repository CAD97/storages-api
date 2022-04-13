use {
    crate::{polyfill::layout_for_metadata, Storage},
    core::{
        alloc::Layout,
        mem::MaybeUninit,
        ptr::{self, Pointee},
    },
};

/// A raw box around some storage. Bundles the storage and its handle.
pub struct RawBox<T: ?Sized, S: Storage> {
    handle: S::Handle,
    metadata: <T as Pointee>::Metadata,
    storage: S,
}

impl<T: ?Sized, S: Storage> RawBox<T, S> {
    fn heap_layout(&self) -> Layout {
        unsafe { layout_for_metadata::<T>(self.metadata).unwrap_unchecked() }
    }

    /// Create a new box for the object described by the given metadata.
    ///
    /// The object is not initialized.
    ///
    /// (Do we want a `new_zeroed`?)
    ///
    /// # Safety
    ///
    /// - The metadata must describe a layout valid for a rust object.
    ///   - This requirement exists due to the safety requirements of
    ///     `size_of_val_raw` and `align_of_val_raw`. I think we definitely want
    ///     a way to go compute layout from type and metadata safely, with a
    ///     check that it produces a valid layout.
    ///   - On each unsized kind, this would imply:
    ///     - slices: size computation uses saturating/checked multiplication.
    ///     - traits: vtable must always be valid (as a safety invariant).
    ///     - composites: size computation uses saturating/checked addition.
    ///     - externs: any valid metadata must compute valid size/align or
    ///       indicates that size/align is unknown and/or needs a valid object.
    ///   - [rust-lang/rust#95832](https://github.com/rust-lang/rust/pull/95832)
    ///     is an attempt to quantize how expensive it would be to make slice
    ///     size computation *always* use saturating math.
    pub unsafe fn new(metadata: <T as Pointee>::Metadata, mut storage: S) -> Result<Self, S> {
        if let Some(layout) = layout_for_metadata::<T>(metadata)
        && let Ok(handle) = storage.allocate(layout)
        {
            Ok(RawBox { handle, metadata, storage })
        } else {
            Err(storage)
        }
    }

    /// Get a reference to the boxed object.
    pub fn as_ref(&self) -> &MaybeUninit<T>
    where
        T: Sized,
    {
        unsafe { &*(self.as_ptr() as *const _) }
    }

    /// Get a pointer valid *for reads only* to the boxed object.
    ///
    /// The pointer is invalidated when the box is moved or used by mutable
    /// reference.
    pub fn as_ptr(&self) -> *const T {
        unsafe {
            let (addr, _) = self
                .storage
                .resolve(self.handle, self.heap_layout())
                .as_ptr()
                .to_raw_parts();
            ptr::from_raw_parts(addr, self.metadata())
        }
    }

    /// Get a mutable reference to the boxed object.
    pub fn as_mut(&mut self) -> &mut MaybeUninit<T>
    where
        T: Sized,
    {
        unsafe { &mut *(self.as_mut_ptr() as *mut _) }
    }

    /// Get a pointer valid for reads and writes to the boxed object.
    ///
    /// The pointer is invalidated when the box is moved or used by reference.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe {
            let (addr, _) = self
                .storage
                .resolve_mut(self.handle, self.heap_layout())
                .as_mut_ptr()
                .to_raw_parts();
            ptr::from_raw_parts_mut(addr, self.metadata())
        }
    }

    /// Get the metadata of the boxed object.
    pub fn metadata(&self) -> <T as Pointee>::Metadata {
        self.metadata
    }
}

unsafe impl<#[may_dangle] T: ?Sized, S: Storage> Drop for RawBox<T, S> {
    fn drop(&mut self) {
        unsafe { self.storage.deallocate(self.handle, self.heap_layout()) }
    }
}
