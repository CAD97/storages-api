use {
    crate::{Handle, Storage},
    core::{alloc::AllocError, ptr::Pointee},
};

pub struct RawBox<T: ?Sized, S: Storage> {
    handle: S::Handle<T>,
    storage: S,
}

impl<T: ?Sized, S: Storage> RawBox<T, S> {
    pub unsafe fn new(meta: <T as Pointee>::Metadata, mut storage: S) -> Result<Self, S> {
        match storage.create(meta) {
            Ok(handle) => Ok(RawBox { handle, storage }),
            Err(AllocError) => Err(storage),
        }
    }

    pub fn as_ptr(&self) -> *const T {
        unsafe { self.storage.resolve(self.handle).as_ptr() }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe { self.storage.resolve_mut(self.handle).as_ptr() }
    }

    pub fn metadata(&self) -> <T as Pointee>::Metadata {
        self.handle.metadata()
    }
}

unsafe impl<#[may_dangle] T: ?Sized, S: Storage> Drop for RawBox<T, S> {
    fn drop(&mut self) {
        unsafe { self.storage.destroy(self.handle) }
    }
}
