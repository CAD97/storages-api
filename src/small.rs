use {
    crate::{AllocStorage, Handle, InlineStorage, InlineStorageHandle, Storage},
    core::{
        alloc::{AllocError, Allocator},
        cmp::Ordering,
        hash::{Hash, Hasher},
        ptr::{NonNull, Pointee},
    },
};

/// A single storage which stores objects inline if it fits, otherwise falling
/// back to using an [`Allocator`].
///
/// The `DataStore` type parameter determines the layout of the inline storage.
/// (It would be nice to use `const LAYOUT: Layout` instead, but the needed
/// features are currently a little *too* incomplete to be usable here.)
pub struct SmallStorage<DataStore, A: Allocator> {
    inline: InlineStorage<DataStore>,
    outline: AllocStorage<A>,
}

pub struct SmallStorageHandle<T: ?Sized> {
    meta: <T as Pointee>::Metadata,
}

impl<DataStore, A: Allocator> SmallStorage<DataStore, A> {
    pub fn new(alloc: A) -> Self {
        Self {
            inline: InlineStorage::new(),
            outline: AllocStorage::new(alloc),
        }
    }
}

unsafe impl<DataStore, A: Allocator, T: ?Sized> Storage<T> for SmallStorage<DataStore, A> {
    type Handle = SmallStorageHandle<T>;

    unsafe fn create(
        &mut self,
        meta: <T as core::ptr::Pointee>::Metadata,
    ) -> Result<Self::Handle, AllocError> {
        if self.inline.fits::<T>(meta) {
            Storage::<T>::create(&mut self.inline, meta)?;
            Ok(SmallStorageHandle { meta })
        } else {
            let (addr, _) = Storage::<T>::create(&mut self.outline, meta)?.to_raw_parts();
            let addr_handle = self.inline.create(())?;
            *self.inline.resolve_mut(addr_handle).as_ptr() = addr;
            Ok(SmallStorageHandle { meta })
        }
    }

    unsafe fn destroy(&mut self, handle: Self::Handle) {
        if self.inline.fits::<T>(handle.meta) {
            Storage::<T>::destroy(&mut self.inline, InlineStorageHandle::new(handle.meta));
        } else {
            let addr_handle = InlineStorageHandle::<NonNull<()>>::new(());
            let addr = *self.inline.resolve(addr_handle).as_ref();
            self.inline.destroy(addr_handle);
            let ptr = NonNull::<T>::from_raw_parts(addr, handle.meta);
            self.outline.destroy(ptr);
        }
    }

    unsafe fn resolve_metadata(&self, handle: Self::Handle) -> <T as Pointee>::Metadata {
        handle.meta
    }

    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<T> {
        let meta = handle.meta;
        if self.inline.fits::<T>(handle.meta) {
            self.inline.resolve(InlineStorageHandle::new(meta))
        } else {
            let addr_handle = InlineStorageHandle::<NonNull<()>>::new(());
            let addr = *self.inline.resolve(addr_handle).as_ref();
            let ptr = NonNull::<T>::from_raw_parts(addr, meta);
            self.outline.resolve(ptr)
        }
    }

    unsafe fn resolve_mut(&mut self, handle: Self::Handle) -> NonNull<T> {
        let meta = handle.meta;
        if self.inline.fits::<T>(handle.meta) {
            self.inline.resolve_mut(InlineStorageHandle::new(meta))
        } else {
            let addr_handle = InlineStorageHandle::<NonNull<()>>::new(());
            let addr = *self.inline.resolve(addr_handle).as_ref();
            let ptr = NonNull::<T>::from_raw_parts(addr, meta);
            self.outline.resolve_mut(ptr)
        }
    }
}

unsafe impl<T: ?Sized> Handle<T> for SmallStorageHandle<T> {}

impl<T: ?Sized> SmallStorageHandle<T> {
    pub fn new(meta: <T as Pointee>::Metadata) -> Self {
        SmallStorageHandle { meta }
    }
}

impl<T: ?Sized> Copy for SmallStorageHandle<T> {}
impl<T: ?Sized> Clone for SmallStorageHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Eq for SmallStorageHandle<T> {}
impl<T: ?Sized> PartialEq for SmallStorageHandle<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.meta == rhs.meta
    }
}

impl<T: ?Sized> PartialOrd for SmallStorageHandle<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl<T: ?Sized> Ord for SmallStorageHandle<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.meta.cmp(&rhs.meta)
    }
}

impl<T: ?Sized> Hash for SmallStorageHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.hash(state)
    }
}
