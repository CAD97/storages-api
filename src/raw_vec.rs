use {
    crate::{Handle, SliceStorage},
    core::alloc::AllocError,
};

/// A raw vec around some slice storage. Bundles the storage and its handle.
///
/// Note that this is *even lower level* than [alloc's `RawVec`] currently. That
/// raw vec handles amortized growth; this raw vec just does exactly as asked.
///
/// [alloc's `RawVec`]: https://github.com/rust-lang/rust/blob/master/library/alloc/src/raw_vec.rs
pub struct RawVec<T, S: SliceStorage> {
    handle: S::Handle<[T]>,
    storage: S,
}

impl<T, S: SliceStorage> RawVec<T, S> {
    /// Create a new empty growable slice in the given storage.
    pub fn new(mut storage: S) -> Result<Self, S> {
        match unsafe { storage.create(0) } {
            Ok(handle) => Ok(Self { handle, storage }),
            Err(AllocError) => Err(storage),
        }
    }

    /// Get a pointer valid *for reads only* to the slice.
    ///
    /// The pointer is invalidated when the vec is moved or used mutably.
    pub fn as_ptr(&self) -> *const [T] {
        unsafe { self.storage.resolve(self.handle).as_ptr() }
    }

    /// Get a pointer valid for reads and writes to the slice.
    ///
    /// The pointer is invalidated when the vec is moved or used mutably.
    pub fn as_mut_ptr(&mut self) -> *mut [T] {
        unsafe { self.storage.resolve_mut(self.handle).as_ptr() }
    }

    /// Get the length of the slice.
    pub fn len(&self) -> usize {
        self.handle.metadata()
    }

    /// Grow the length of the slice to `new_len`. Does not change the length
    /// if the slice is already long enough. Does not do amortization.
    pub fn grow_to(&mut self, new_len: usize) -> Result<(), AllocError> {
        if new_len <= self.len() {
            Ok(())
        } else {
            self.handle = unsafe { self.storage.grow(self.handle, new_len) }?;
            Ok(())
        }
    }

    /// Shrink the length of the slice to `new_len`. Does not change the length
    /// if the slice is already shorter than the given length.
    pub fn shrink_to(&mut self, new_len: usize) -> Result<(), AllocError> {
        if new_len >= self.len() {
            Ok(())
        } else {
            self.handle = unsafe { self.storage.shrink(self.handle, new_len) }?;
            Ok(())
        }
    }
}

unsafe impl<#[may_dangle] T, S: SliceStorage> Drop for RawVec<T, S> {
    fn drop(&mut self) {
        unsafe { self.storage.destroy(self.handle) }
    }
}
