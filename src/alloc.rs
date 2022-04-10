use {
    crate::{
        polyfill::layout_for_meta, MultipleStorage, PinningStorage, SharedMutabilityStorage,
        SliceStorage, Storage,
    },
    core::{
        alloc::{AllocError, Allocator, Layout},
        ptr::{metadata, NonNull, Pointee},
    },
};

/// A storage that stores objects via an [`Allocator`].
pub struct AllocStorage<A: Allocator> {
    alloc: A,
}

impl<A: Allocator> AllocStorage<A> {
    pub fn new(alloc: A) -> Self {
        Self { alloc }
    }
}

unsafe impl<A: Allocator, T: ?Sized> SharedMutabilityStorage<T> for AllocStorage<A> {}
unsafe impl<A: Allocator, T: ?Sized> MultipleStorage<T> for AllocStorage<A> {}
unsafe impl<A: Allocator, T: ?Sized> PinningStorage<T> for AllocStorage<A> {}
unsafe impl<A: Allocator, T: ?Sized> Storage<T> for AllocStorage<A> {
    type Handle = NonNull<T>;

    unsafe fn create(
        &mut self,
        meta: <T as Pointee>::Metadata,
    ) -> Result<Self::Handle, AllocError> {
        let layout = layout_for_meta::<T>(meta).ok_or(AllocError)?;
        let ptr = self.alloc.allocate(layout)?;
        Ok(NonNull::from_raw_parts(ptr.cast(), meta))
    }

    unsafe fn destroy(&mut self, handle: Self::Handle) {
        let layout = Layout::for_value_raw(handle.as_ptr());
        self.alloc.deallocate(handle.cast(), layout)
    }

    unsafe fn resolve_metadata(&self, handle: Self::Handle) -> <T as Pointee>::Metadata {
        metadata(handle.as_ptr())
    }

    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<T> {
        handle
    }

    unsafe fn resolve_mut(&mut self, handle: Self::Handle) -> NonNull<T> {
        handle
    }
}

unsafe impl<A: Allocator, T> SliceStorage<T> for AllocStorage<A> {
    unsafe fn grow(
        &mut self,
        handle: Self::Handle,
        new_len: usize,
    ) -> Result<Self::Handle, AllocError> {
        let meta = self.resolve_metadata(handle);
        let old_layout = Layout::for_value_raw(handle.as_ptr());
        let new_layout = Layout::array::<T>(new_len).map_err(|_| AllocError)?;
        let ptr = self.alloc.grow(handle.cast(), old_layout, new_layout)?;
        Ok(NonNull::from_raw_parts(ptr.cast(), meta))
    }

    unsafe fn shrink(
        &mut self,
        handle: Self::Handle,
        new_len: usize,
    ) -> Result<Self::Handle, AllocError> {
        let meta = self.resolve_metadata(handle);
        let old_layout = Layout::for_value_raw(handle.as_ptr());
        let new_layout = Layout::array::<T>(new_len).map_err(|_| AllocError)?;
        let ptr = self.alloc.shrink(handle.cast(), old_layout, new_layout)?;
        Ok(NonNull::from_raw_parts(ptr.cast(), meta))
    }
}
