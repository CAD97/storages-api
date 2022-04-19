use core::{
    alloc::{AllocError, Layout},
    hash::Hash,
    mem::MaybeUninit,
    ptr::copy_nonoverlapping,
};

pub type Memory = [MaybeUninit<u8>];

/// Types which can be used to manage memory handles.
///
/// The behavior of this trait is refined by traits [`PinningStorage`],
/// [`MultipleStorage`], and [`SharedMutabilityStorage`].
pub unsafe trait Storage {
    /// The handle which is used to access the stored memory.
    ///
    /// (This should probably also include `Send + Sync`, as only the Send/Sync
    /// of the Storage matters, not the handles themselves, but the simplicity
    /// of being able to use `ptr::NonNull<()>` as a handle is appealing.)
    type Handle: Copy + Ord + Hash + Unpin;

    /// Allocate memory handle in this storage.
    ///
    /// The handled memory is not initialized. Any existing handles are
    /// invalidated.
    ///
    /// (Do we want an `allocate_zeroed`?)
    fn allocate(&mut self, layout: Layout) -> Result<Self::Handle, AllocError>;

    /// Deallocate an object handle in this storage.
    ///
    /// The handled memory is not required to be valid in any way. The handle is
    /// invalidated.
    ///
    /// # Safety
    ///
    /// - The handle must have been created by this storage, and must not have
    ///   been invalidated.
    /// - The layout must be the same as used to allocate the handle.
    unsafe fn deallocate(&mut self, handle: Self::Handle, layout: Layout);

    /// Resolve a memory handle in this storage to a reference.
    ///
    /// # Safety
    ///
    /// - The handle must have been created by this storage, and must not have
    ///   been invalidated.
    /// - The layout must be the same as used to allocate the handle.
    unsafe fn resolve(&self, handle: Self::Handle, layout: Layout) -> &Memory;

    /// Resolve a memory handle in this storage to a mutable reference.
    ///
    /// # Safety
    ///
    /// - The handle must have been created by this storage, and must not have
    ///   been invalidated.
    /// - The layout must be the same as used to allocate the handle.
    unsafe fn resolve_mut(&mut self, handle: Self::Handle, layout: Layout) -> &mut Memory;

    /// Grow a memory handle to a larger size.
    ///
    /// If this function succeeds, then the old handle is invalidated and the
    /// handled memory has been moved into the new handle. The new length is
    /// uninitialized.
    ///
    /// (Do we want a `grow_zeroed`?)
    ///
    /// If this function fails, then the old handle is not invalidated and
    /// still contains the memory in its state before calling this function.
    ///
    /// # Safety
    ///
    /// - The handle must have been created by this storage, and must not have
    ///   been invalidated.
    /// - `old_layout` must be the same as used to allocate the handle.
    /// - `new_layout.size() >= old_layout.size()`.
    ///
    /// Note that `new_layout.align()` is not required to be the same as
    /// `old_layout.align()`
    unsafe fn grow(
        &mut self,
        handle: Self::Handle,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError>;

    /// Shrink a memory handle to a smaller size.
    ///
    /// If this function succeeds, then the old handle is invalidated and the
    /// prefix of the handled memory has been moved into the new handle.
    ///
    /// If this function fails, then the old handle is not invalidated and
    /// still contains the memory in its state before calling this function.
    ///
    /// # Safety
    ///
    /// - The handle must have been created by this storage, and must not have
    ///   been invalidated.
    /// - `old_layout` must be the same as used to allocate the handle.
    /// - `new_layout.size() <= old_layout.size()`.
    ///
    /// Note that `new_layout.align()` is not required to be the same as
    /// `old_layout.align()`
    unsafe fn shrink(
        &mut self,
        handle: Self::Handle,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError>;
}

/// A storage that allocates pinned memory handles.
///
/// Any memory allocated inside of this storage will not be moved nor reused
/// until [`deallocate`] is called on its handle. As such, an object in the
/// memory can be safely pinned.
///
/// [`deallocate`]: Storage::deallocate
pub unsafe trait PinningStorage: Storage {}

/// A storage that can manage multiple memory handles.
///
/// [`allocate`] no longer invalidates existing handles. You can `allocate`
/// multiple handles that are all valid at the same time. This does not change
/// the requirements of [`resolve`] or [`resolve_mut`]; only one pointer
/// returned from `resolve_mut` is valid at a time.
///
/// In addition, [`grow`] and [`shrink`] are implemented at least at least as
/// well as creating a new handle and copying over the old contents. (This is
/// provided by default impl.)
///
/// [`allocate`]: Storage::allocate
/// [`resolve`]: Storage::resolve
/// [`resolve_mut`]: Storage::resolve_mut
/// [`grow`]: Storage::grow
/// [`shrink`]: Storage::shrink
pub unsafe trait MultipleStorage: Storage {
    /// Resolve memory handles in this storage to mutable references.
    ///
    /// # Safety
    ///
    /// - The handles must have been created by this storage, and must not have
    ///   been invalidated.
    /// - The layout must be the same as used to allocate the handle.
    /// - The same handle must not be resolved twice in a single call.
    unsafe fn resolve_many_mut<const N: usize>(
        &mut self,
        handles: [(Self::Handle, Layout); N],
    ) -> [&mut Memory; N];
}

/// A storage that serves as a uniqueness barrier.
///
/// Notably, this means that this storage can go `&Storage -> &mut Memory`, and
/// thus it is possible to mutate the stored memory behind a shared storage
/// reference, and to mutably resolve multiple handles separately without
/// invalidating previously resolved handles.
///
/// [`resolve`]: Storage::resolve
/// [`resolve_mut`]: Storage::resolve_mut
pub unsafe trait SharedMutabilityStorage: Storage {
    /// Resolve a memory handle in this storage to a mutable reference.
    ///
    /// # Safety
    ///
    /// - The handle must have been created by this storage, and must not have
    ///   been invalidated.
    /// - The layout must be the same as used to allocate the handle.
    unsafe fn resolve_raw(&self, handle: Self::Handle, layout: Layout) -> &mut Memory;
}

default unsafe impl<S> Storage for S
where
    S: MultipleStorage,
{
    default unsafe fn grow(
        &mut self,
        old_handle: Self::Handle,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "invalid arguments to Storage::grow",
        );

        let new_handle: Self::Handle = self.allocate(new_layout)?;
        let [new_ptr, old_ptr] =
            self.resolve_many_mut([(new_handle, new_layout), (old_handle, old_layout)]);

        copy_nonoverlapping(
            old_ptr.as_mut_ptr(),
            new_ptr.as_mut_ptr(),
            old_layout.size(),
        );

        self.deallocate(old_handle, old_layout);
        Ok(new_handle)
    }

    default unsafe fn shrink(
        &mut self,
        old_handle: Self::Handle,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "invalid arguments to Storage::shrink",
        );

        let new_handle: Self::Handle = self.allocate(new_layout)?;
        let [new_ptr, old_ptr] =
            self.resolve_many_mut([(new_handle, new_layout), (old_handle, old_layout)]);

        copy_nonoverlapping(
            old_ptr.as_mut_ptr(),
            new_ptr.as_mut_ptr(),
            new_layout.size(),
        );

        self.deallocate(old_handle, old_layout);
        Ok(new_handle)
    }
}
