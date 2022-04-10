use {
    crate::Storage,
    core::{
        alloc::AllocError,
        mem::ManuallyDrop,
        ptr::{self, Pointee},
    },
};

/// A raw box around some storage. Bundles the storage and its handle.
pub struct RawBox<T: ?Sized, S: Storage<T>> {
    handle: S::Handle,
    storage: S,
}

impl<T: ?Sized, S: Storage<T>> RawBox<T, S> {
    /// Create a new box for the object described by the given metadata.
    ///
    /// The object is not initialized.
    ///
    /// (Do we want a `new_zeroed`?)
    ///
    /// # Safety
    ///
    /// - The metadata must describe a layout valid for a rust object.
    ///   - This exists due to the safety requirements of `size_of_val_raw`
    ///     and `align_of_val_raw`. I think we need a way to go compute layout
    ///     from metadata safely, with a check that it produces a valid layout.
    ///   - On each unsized kind, this would imply:
    ///     - slices: size computation uses saturating/checked multiplication
    ///     - traits: vtable must always be valid (as a safety invariant)
    ///     - composites: size computation uses saturating/checked addition
    ///     - externs: any valid metadata must compute valid size/align or None
    pub unsafe fn new(meta: <T as Pointee>::Metadata, mut storage: S) -> Result<Self, S> {
        match storage.create(meta) {
            Ok(handle) => Ok(RawBox { handle, storage }),
            Err(AllocError) => Err(storage),
        }
    }

    pub unsafe fn from_raw_parts(handle: S::Handle, storage: S) -> Self {
        Self { handle, storage }
    }

    pub fn into_raw_parts(self) -> (S::Handle, S) {
        let this = ManuallyDrop::new(self);
        (this.handle, unsafe { ptr::read(&this.storage) })
    }

    /// Get a pointer valid *for reads only* to the object.
    ///
    /// The pointer is invalidated when the box is moved or used by mutable
    /// reference.
    pub fn as_ptr(&self) -> *const T {
        unsafe { self.storage.resolve(self.handle).as_ptr() }
    }

    /// Get a pointer valid for reads and writes to the object.
    ///
    /// The pointer is invalidated when the box is moved or used by reference.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { self.storage.resolve_mut(self.handle).as_ptr() }
    }

    /// Get the metadata of the inner object.
    pub fn metadata(&self) -> <T as Pointee>::Metadata {
        unsafe { self.storage.resolve_metadata(self.handle) }
    }
}

unsafe impl<#[may_dangle] T: ?Sized, S: Storage<T>> Drop for RawBox<T, S> {
    fn drop(&mut self) {
        unsafe { self.storage.destroy(self.handle) }
    }
}
