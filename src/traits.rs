use core::{
    alloc::AllocError,
    ptr::{copy_nonoverlapping, metadata, NonNull, Pointee},
};

pub unsafe trait Handle<T: ?Sized>: Copy {
    fn metadata(self) -> <T as Pointee>::Metadata;
}

pub unsafe trait Storage {
    type Handle<T: ?Sized>: Handle<T>;

    unsafe fn create<T: ?Sized>(
        &mut self,
        meta: <T as Pointee>::Metadata,
    ) -> Result<Self::Handle<T>, AllocError>;

    unsafe fn destroy<T: ?Sized>(&mut self, handle: Self::Handle<T>);

    unsafe fn resolve<T: ?Sized>(&self, handle: Self::Handle<T>) -> NonNull<T>;
    unsafe fn resolve_mut<T: ?Sized>(&mut self, handle: Self::Handle<T>) -> NonNull<T>;
}

pub unsafe trait PinningStorage: Storage {}
pub unsafe trait MultipleStorage: Storage {}
pub unsafe trait SharedMutabilityStorage: Storage {}

pub unsafe trait SliceStorage: Storage {
    unsafe fn grow<T>(
        &mut self,
        handle: Self::Handle<[T]>,
        new_len: usize,
    ) -> Result<Self::Handle<[T]>, AllocError>;

    unsafe fn shrink<T>(
        &mut self,
        handle: Self::Handle<[T]>,
        new_len: usize,
    ) -> Result<Self::Handle<[T]>, AllocError>;
}

default unsafe impl<S: MultipleStorage> SliceStorage for S {
    default unsafe fn grow<T>(
        &mut self,
        old_handle: Self::Handle<[T]>,
        new_len: usize,
    ) -> Result<Self::Handle<[T]>, AllocError> {
        let new_handle: Self::Handle<[T]> = self.create(new_len)?;
        let new_ptr: NonNull<[T]> = self.resolve_mut(new_handle);

        let old_ptr: NonNull<[T]> = self.resolve_mut(old_handle);
        let old_len: usize = metadata(old_ptr.as_ptr());

        debug_assert!(new_len >= old_len, "invalid arguments to Storage::grow");

        copy_nonoverlapping(
            old_ptr.as_ptr().cast::<T>(),
            new_ptr.as_ptr().cast::<T>(),
            old_len,
        );

        self.destroy(old_handle);
        Ok(new_handle)
    }

    default unsafe fn shrink<T>(
        &mut self,
        old_handle: Self::Handle<[T]>,
        new_len: usize,
    ) -> Result<Self::Handle<[T]>, AllocError> {
        let new_handle: Self::Handle<[T]> = self.create(new_len)?;
        let new_ptr: NonNull<[T]> = self.resolve_mut(new_handle);

        let old_ptr: NonNull<[T]> = self.resolve_mut(old_handle);
        let old_len: usize = metadata(old_ptr.as_ptr());

        debug_assert!(new_len <= old_len, "invalid arguments to Storage::grow");

        copy_nonoverlapping(
            old_ptr.as_ptr().cast::<T>(),
            new_ptr.as_ptr().cast::<T>(),
            old_len,
        );

        self.destroy(old_handle);
        Ok(new_handle)
    }
}

unsafe impl<T: ?Sized> Handle<T> for NonNull<T> {
    fn metadata(self) -> <T as Pointee>::Metadata {
        metadata(self.as_ptr())
    }
}
