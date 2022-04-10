use core::{
    alloc::AllocError,
    mem::MaybeUninit,
    ptr::{copy_nonoverlapping, metadata, NonNull, Pointee},
};

/// Types which can be used as a storage handle.
///
/// Generically, all you can do is extract the metadata or give it to the
/// storage. Some other operations that might be useful/necessary to have in
/// the future:
///
/// - `cast`
/// - `unsize`
/// - `with_metadata_of`
pub unsafe trait Handle<T: ?Sized>: Copy {
    fn metadata(self) -> <T as Pointee>::Metadata;
}

/// Types which can be used to store objects.
///
/// The behavior of this trait is refined by traits [`PinningStorage`],
/// [`MultipleStorage`], and [`SharedMutabilityStorage`].
///
/// (I've used `create`/`destroy` to clearly separate this from `Allocator`'s
/// `allocate`/`deallocate` language, but naming is to be bikeshedded further.)
pub unsafe trait Storage {
    /// The handle which is used to access
    type Handle<T: ?Sized>: Handle<T>;

    /// Create an object handle in this storage.
    ///
    /// The handled object is not initialized.
    ///
    /// (Do we want a `create_zeroed`?)
    ///
    /// # Safety
    ///
    /// - Any previously created handles have been destroyed.
    ///   - (This can maybe be loosened to it invalidating existing handles?)
    /// - The metadata must describe a layout valid for a rust object.
    ///   - This exists due to the safety requirements of `size_of_val_raw`
    ///     and `align_of_val_raw`. I think we need a way to go compute layout
    ///     from metadata safely, with a check that it produces a valid layout.
    ///   - On each unsized kind, this would imply:
    ///     - slices: size computation uses saturating/checked multiplication
    ///     - traits: vtable must always be valid (as a safety invariant)
    ///     - composites: size computation uses saturating/checked addition
    ///     - externs: any valid metadata must compute valid size/align or None
    unsafe fn create<T: ?Sized>(
        &mut self,
        meta: <T as Pointee>::Metadata,
    ) -> Result<Self::Handle<T>, AllocError>;

    /// Destroy an object handle in this storage.
    ///
    /// The handled object is not modified nor required to be valid in any way.
    ///
    /// # Safety
    ///
    /// - The handle must have previously been created by this storage,
    ///   and must not have been destroyed.
    unsafe fn destroy<T: ?Sized>(&mut self, handle: Self::Handle<T>);

    /// Resolve an object handle in this storage to a pointer.
    ///
    /// The returned pointer is valid *for reads only* and is invalidated
    /// when the storage is moved or used mutably.
    ///
    /// # Safety
    ///
    /// - The handle must have previously been created by this storage,
    ///   and must not have been destroyed or invalidated.
    unsafe fn resolve<T: ?Sized>(&self, handle: Self::Handle<T>) -> NonNull<T>;

    /// Resolve an object handle in this storage to a pointer.
    ///
    /// The returned pointer is valid for both reads and writes and is
    /// invalidated when the storage is moved or used mutably. (This includes
    /// but is not limited to further calls to `resolve_mut`.)
    ///
    /// # Safety
    ///
    /// - The handle must have previously been created by this storage,
    ///   and must not have been destroyed or invalidated.
    unsafe fn resolve_mut<T: ?Sized>(&mut self, handle: Self::Handle<T>) -> NonNull<T>;
}

/// A storage that creates pinned handles.
///
/// Any objects created inside of this storage will not be moved and will not
/// be deallocated until [`destroy`][Self::destroy] is called on their handle.
pub unsafe trait PinningStorage: Storage {}

/// A storage that can create multiple handles.
///
/// The restriction that [`create`] can not be called until the previously
/// created handle has been [`destroy`]ed is removed. You can `create` multiple
/// handles that are all valid at the same time. This does not change the
/// requirements of [`resolve`] or [`resolve_mut`]; only one pointer returned
/// from `resolve_mut` is valid at a time.
///
/// (In theory, we're missing the ability to `create` multiple handles to a
/// `!SharedMutabilityStorage` and access their objects mutably simultaneously.
/// This is possible to do soundly (for the homogeneously-typed case, store a
/// slice/array and `split_at_mut` it), but seems rare enough in practice that
/// the complexity to support it doesn't seem warranted? Such a function would
/// live on this trait and could look like the following:
///
/// ```rust,compile_fail
/// unsafe fn resolve_many_mut<T: ?Sized, N: usize>(
///     &mut self,
///     handles: [Self::Handle<T>; N],
/// ) -> [NonNull<T>; N];
/// ```
///
/// The reason for this restriction is that reborrowing `&mut Storage` to call
/// `resolve_mut` invalidates any references/pointers derived from the previous
/// reborrow of `&mut Storage`. This is fundamental to `&mut` being `noalias`.)
///
/// [`create`]: Self::create
/// [`destroy`]: Self::destroy
/// [`resolve`]: Self::resolve
/// [`resolve_mut`]: Self::resolve_mut
pub unsafe trait MultipleStorage: Storage {}

/// A storage that serves as a uniqueness barrier.
///
/// Pointers returned from [`resolve`][Self::resolve] are valid for writes.
pub unsafe trait SharedMutabilityStorage: Storage {}

/// A storage that can reallocate to adjust the length of slice objects.
///
/// Automatically provided for any [`MultipleStorage`] by allocating a new
/// object and copying the old allocation into the new one.
pub unsafe trait SliceStorage: Storage {
    /// Grow a slice handle to a larger size.
    ///
    /// If this function succeeds, then the old handle is invalidated and the
    /// handled object has been moved into the new handle. The new length is
    /// uninitialized.
    ///
    /// (Do we want a `grow_zeroed`?)
    ///
    /// If this function fails, then the old handle is not invalidated and
    /// still contains the object.
    ///
    /// # Safety
    ///
    /// - The handle must have previously been created by this storage,
    ///   and must not have been destroyed or invalidated.
    /// - `new_len` must be longer than the existing slice length.
    unsafe fn grow<T>(
        &mut self,
        handle: Self::Handle<[T]>,
        new_len: usize,
    ) -> Result<Self::Handle<[T]>, AllocError>;

    /// Shrink a slice handle to a smaller size.
    ///
    /// If this function succeeds, then the old handle is invalidated and the
    /// prefix of the handled object has been moved into the new handle.
    ///
    /// If this function fails, then the old handle is not invalidated and
    /// still contains the object.
    ///
    /// # Safety
    ///
    /// - The handle must have previously been created by this storage,
    ///   and must not have been destroyed or invalidated.
    /// - `new_len` must be shorter than the existing slice length.
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
            old_ptr.as_ptr().cast::<MaybeUninit<T>>(),
            new_ptr.as_ptr().cast::<MaybeUninit<T>>(),
            new_len,
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
