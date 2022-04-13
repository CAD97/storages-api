use {
    crate::{polyfill::layout_for_metadata, Storage},
    core::{
        alloc::{AllocError, Layout},
        mem::MaybeUninit,
        ptr::{self, Pointee},
    },
};

/// A raw vec around some slice storage. Bundles the storage and its handle.
///
/// Note that this is *even lower level* than [alloc's `RawVec`] currently. That
/// raw vec handles amortized growth; this raw vec just does exactly as asked.
///
/// [alloc's `RawVec`]: https://github.com/rust-lang/rust/blob/master/library/alloc/src/raw_vec.rs
pub struct RawVec<T, S: Storage> {
    handle: S::Handle,
    metadata: <[T] as Pointee>::Metadata,
    storage: S,
}

impl<T, S: Storage> RawVec<T, S> {
    fn heap_layout(&self) -> Layout {
        Self::heap_layout_for(self.len())
    }

    fn heap_layout_for(len: usize) -> Layout {
        unsafe { layout_for_metadata::<[T]>(len).unwrap_unchecked() }
    }

    /// Create a new empty growable slice in the given storage.
    pub fn new(mut storage: S) -> Result<Self, S> {
        if let Ok(handle) = storage.allocate(Self::heap_layout_for(0)) {
            Ok(Self {
                handle,
                metadata: 0,
                storage,
            })
        } else {
            Err(storage)
        }
    }

    /// Get a reference to the boxed slice.
    pub fn as_ref(&self) -> &[MaybeUninit<T>] {
        unsafe {
            let (addr, _) = self
                .storage
                .resolve(self.handle, self.heap_layout())
                .as_ptr()
                .to_raw_parts();
            &*ptr::from_raw_parts(addr, self.len())
        }
    }

    /// Get a mutable reference to the boxed slice.
    pub fn as_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe {
            let (addr, _) = self
                .storage
                .resolve_mut(self.handle, self.heap_layout())
                .as_mut_ptr()
                .to_raw_parts();
            &mut *ptr::from_raw_parts_mut(addr, self.len())
        }
    }

    /// Get the length of the slice.
    pub fn len(&self) -> usize {
        self.metadata
    }

    /// Grow the length of the slice to `new_len`. Does not change the length
    /// if the slice is already long enough. Does not do amortization.
    pub fn grow_to(&mut self, new_len: usize) -> Result<(), AllocError> {
        if new_len <= self.len() {
            Ok(())
        } else {
            self.handle = unsafe {
                self.storage.grow(
                    self.handle,
                    self.heap_layout(),
                    Self::heap_layout_for(new_len),
                )
            }?;
            self.metadata = new_len;
            Ok(())
        }
    }

    /// Shrink the length of the slice to `new_len`. Does not change the length
    /// if the slice is already shorter than the given length.
    pub fn shrink_to(&mut self, new_len: usize) -> Result<(), AllocError> {
        if new_len >= self.len() {
            Ok(())
        } else {
            self.handle = unsafe {
                self.storage.shrink(
                    self.handle,
                    self.heap_layout(),
                    Self::heap_layout_for(new_len),
                )
            }?;
            Ok(())
        }
    }
}

unsafe impl<#[may_dangle] T, S: Storage> Drop for RawVec<T, S> {
    fn drop(&mut self) {
        unsafe { self.storage.deallocate(self.handle, self.heap_layout()) }
    }
}
