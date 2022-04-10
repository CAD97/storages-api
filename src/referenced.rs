use {
    crate::{Handle, Storage},
    core::{
        alloc::AllocError,
        cmp::Ordering,
        hash::{Hash, Hasher},
        mem::ManuallyDrop,
        ptr::{metadata, NonNull, Pointee},
    },
};

/// A storage wrapper around `&mut T`.
///
/// This storage cannot be used to [`create`][Self::create] new handles.
/// Instead, when created, there is a single handle to the contents of the
/// wrapped reference. Attempting to [`destroy`][Self::destroy] this handle
/// is a no-op.
pub struct RefStorage<'a, T: ?Sized> {
    data: &'a mut T,
}

/// A handle into a [`RefStorage]`.
pub struct RefStorageHandle;

impl<'a, T: ?Sized> RefStorage<'a, T> {
    /// Create a new borrowed storage.
    pub fn new(data: &'a mut T) -> Self {
        Self { data }
    }

    /// Get the wrapped reference without going through a storage handle.
    pub fn get(&self) -> &T {
        self.data
    }

    /// Get the wrapped reference without going through a storage handle.
    pub fn get_mut(&mut self) -> &mut T {
        self.data
    }
}

unsafe impl<T: ?Sized> Storage<T> for RefStorage<'_, T> {
    type Handle = RefStorageHandle;

    /// Always returns an error. See the type docs for more info.
    unsafe fn create(
        &mut self,
        _meta: <T as core::ptr::Pointee>::Metadata,
    ) -> Result<Self::Handle, AllocError> {
        Err(AllocError)
    }

    // No-op. See the type docs for more info.
    unsafe fn destroy(&mut self, _handle: Self::Handle) {}

    unsafe fn resolve_metadata(&self, _handle: Self::Handle) -> <T as Pointee>::Metadata {
        metadata(self.data)
    }

    unsafe fn resolve(&self, _handle: Self::Handle) -> NonNull<T> {
        NonNull::from(self.get())
    }

    unsafe fn resolve_mut(&mut self, _handle: Self::Handle) -> NonNull<T> {
        NonNull::from(self.get_mut())
    }
}

unsafe impl<T: ?Sized> Storage<T> for RefStorage<'_, ManuallyDrop<T>> {
    type Handle = RefStorageHandle;

    /// Always returns an error. See the type docs for more info.
    unsafe fn create(
        &mut self,
        _meta: <T as core::ptr::Pointee>::Metadata,
    ) -> Result<Self::Handle, AllocError> {
        Err(AllocError)
    }

    // No-op. See the type docs for more info.
    unsafe fn destroy(&mut self, _handle: Self::Handle) {}

    unsafe fn resolve_metadata(&self, _handle: Self::Handle) -> <T as Pointee>::Metadata {
        metadata(self.data as *const ManuallyDrop<T> as *mut T)
    }

    unsafe fn resolve(&self, _handle: Self::Handle) -> NonNull<T> {
        NonNull::new_unchecked(self.data as *const ManuallyDrop<T> as *mut T)
    }

    unsafe fn resolve_mut(&mut self, _handle: Self::Handle) -> NonNull<T> {
        NonNull::new_unchecked(self.data as *mut ManuallyDrop<T> as *mut T)
    }
}

unsafe impl<T: ?Sized> Handle<T> for RefStorageHandle {}

impl RefStorageHandle {
    pub fn new() -> Self {
        RefStorageHandle
    }
}

impl Copy for RefStorageHandle {}
impl Clone for RefStorageHandle {
    fn clone(&self) -> Self {
        *self
    }
}

impl Eq for RefStorageHandle {}
impl PartialEq for RefStorageHandle {
    fn eq(&self, _rhs: &Self) -> bool {
        true
    }
}

impl PartialOrd for RefStorageHandle {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for RefStorageHandle {
    fn cmp(&self, _rhs: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl Hash for RefStorageHandle {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
