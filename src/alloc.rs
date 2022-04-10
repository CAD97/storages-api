use {
    crate::{
        polyfill::layout_for_meta, Handle, MultipleStorage, PinningStorage,
        SharedMutabilityStorage, SliceStorage, Storage,
    },
    core::{
        alloc::{AllocError, Allocator, Layout},
        ptr::{NonNull, Pointee},
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

unsafe impl<A: Allocator> SharedMutabilityStorage for AllocStorage<A> {}
unsafe impl<A: Allocator> MultipleStorage for AllocStorage<A> {}
unsafe impl<A: Allocator> PinningStorage for AllocStorage<A> {}
unsafe impl<A: Allocator> Storage for AllocStorage<A> {
    type Handle<T: ?Sized> = NonNull<T>;

    unsafe fn create<T: ?Sized>(
        &mut self,
        meta: <T as Pointee>::Metadata,
    ) -> Result<Self::Handle<T>, AllocError> {
        let layout = layout_for_meta::<T>(meta).ok_or(AllocError)?;
        let ptr = self.alloc.allocate(layout)?;
        Ok(NonNull::from_raw_parts(ptr.cast(), meta))
    }

    unsafe fn destroy<T: ?Sized>(&mut self, handle: Self::Handle<T>) {
        let layout = Layout::for_value_raw(handle.as_ptr());
        self.alloc.deallocate(handle.cast(), layout)
    }

    unsafe fn resolve<T: ?Sized>(&self, handle: Self::Handle<T>) -> NonNull<T> {
        handle
    }

    unsafe fn resolve_mut<T: ?Sized>(&mut self, handle: Self::Handle<T>) -> NonNull<T> {
        handle
    }
}

unsafe impl<A: Allocator> SliceStorage for AllocStorage<A> {
    unsafe fn grow<T>(
        &mut self,
        handle: Self::Handle<[T]>,
        new_len: usize,
    ) -> Result<Self::Handle<[T]>, AllocError> {
        let meta = handle.metadata();
        let old_layout = Layout::for_value_raw(handle.as_ptr());
        let new_layout = Layout::array::<T>(new_len).map_err(|_| AllocError)?;
        let ptr = self.alloc.grow(handle.cast(), old_layout, new_layout)?;
        Ok(NonNull::from_raw_parts(ptr.cast(), meta))
    }

    unsafe fn shrink<T>(
        &mut self,
        handle: Self::Handle<[T]>,
        new_len: usize,
    ) -> Result<Self::Handle<[T]>, AllocError> {
        let meta = handle.metadata();
        let old_layout = Layout::for_value_raw(handle.as_ptr());
        let new_layout = Layout::array::<T>(new_len).map_err(|_| AllocError)?;
        let ptr = self.alloc.shrink(handle.cast(), old_layout, new_layout)?;
        Ok(NonNull::from_raw_parts(ptr.cast(), meta))
    }
}
