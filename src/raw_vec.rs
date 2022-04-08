use {
    crate::{Handle, SliceStorage},
    core::alloc::AllocError,
};

pub struct RawVec<T, S: SliceStorage> {
    handle: S::Handle<[T]>,
    storage: S,
}

impl<T, S: SliceStorage> RawVec<T, S> {
    pub fn new(mut storage: S) -> Result<Self, S> {
        match unsafe { storage.create(0) } {
            Ok(handle) => Ok(Self { handle, storage }),
            Err(AllocError) => Err(storage),
        }
    }

    pub fn as_ptr(&self) -> *const [T] {
        unsafe { self.storage.resolve(self.handle).as_ptr() }
    }

    pub fn as_mut_ptr(&mut self) -> *mut [T] {
        unsafe { self.storage.resolve_mut(self.handle).as_ptr() }
    }

    pub fn len(&self) -> usize {
        self.handle.metadata()
    }

    pub fn grow_to(&mut self, new_len: usize) -> Result<(), AllocError> {
        if new_len <= self.len() {
            Ok(())
        } else {
            self.handle = unsafe { self.storage.grow(self.handle, new_len) }?;
            Ok(())
        }
    }

    pub fn shrink_to(&mut self, new_len: usize) -> Result<(), AllocError> {
        if new_len <= self.len() {
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
