use {
    crate::{
        polyfill::{handle_alloc_error, layout_for_metadata},
        Storage,
    },
    core::{
        alloc::Layout,
        mem::{ManuallyDrop, MaybeUninit},
        ops::{Deref, DerefMut},
        ptr::{self, Pointee},
    },
    unsize::CoerciblePtr,
};

/// A raw box around some storage. Bundles the storage and its handle.
pub struct RawBox<T: ?Sized, S: Storage> {
    handle: S::Handle,
    metadata: <T as Pointee>::Metadata,
    storage: S,
}

impl<T: ?Sized, S: Storage> RawBox<T, S> {
    fn heap_layout(&self) -> Layout {
        unsafe { layout_for_metadata::<T>(self.metadata).unwrap_unchecked() }
    }

    /// Create a new box for the object described by the given metadata.
    ///
    /// The object is not initialized.
    ///
    /// (Do we want a `new_zeroed`?)
    ///
    /// # Safety
    ///
    /// - The metadata must describe a layout valid for a rust object.
    ///   - This requirement exists due to the safety requirements of
    ///     `size_of_val_raw` and `align_of_val_raw`. I think we definitely want
    ///     a way to go compute layout from type and metadata safely, with a
    ///     check that it produces a valid layout.
    ///   - On each unsized kind, this would imply:
    ///     - slices: size computation uses saturating/checked multiplication.
    ///     - traits: vtable must always be valid (as a safety invariant).
    ///     - composites: size computation uses saturating/checked addition.
    ///     - externs: any valid metadata must compute valid size/align or
    ///       indicates that size/align is unknown and/or needs a valid object.
    ///   - [rust-lang/rust#95832](https://github.com/rust-lang/rust/pull/95832)
    ///     is an attempt to quantize how expensive it would be to make slice
    ///     size computation *always* use saturating math.
    pub unsafe fn new(metadata: <T as Pointee>::Metadata, mut storage: S) -> Result<Self, S> {
        if let Some(layout) = layout_for_metadata::<T>(metadata)
        && let Ok(handle) = storage.allocate(layout)
        {
            Ok(RawBox { handle, metadata, storage })
        } else {
            Err(storage)
        }
    }

    /// Get a reference to the boxed object.
    pub fn as_ref(&self) -> &MaybeUninit<T>
    where
        T: Sized,
    {
        unsafe { &*(self.as_ptr() as *const _) }
    }

    /// Get a pointer valid *for reads only* to the boxed object.
    ///
    /// The pointer is invalidated when the box is moved or used by mutable
    /// reference.
    pub fn as_ptr(&self) -> *const T {
        unsafe {
            let (addr, _) = self
                .storage
                .resolve(self.handle, self.heap_layout())
                .as_ptr()
                .to_raw_parts();
            ptr::from_raw_parts(addr, self.metadata())
        }
    }

    /// Get a mutable reference to the boxed object.
    pub fn as_mut(&mut self) -> &mut MaybeUninit<T>
    where
        T: Sized,
    {
        unsafe { &mut *(self.as_mut_ptr() as *mut _) }
    }

    /// Get a pointer valid for reads and writes to the boxed object.
    ///
    /// The pointer is invalidated when the box is moved or used by reference.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe {
            let (addr, _) = self
                .storage
                .resolve_mut(self.handle, self.heap_layout())
                .as_mut_ptr()
                .to_raw_parts();
            ptr::from_raw_parts_mut(addr, self.metadata())
        }
    }

    /// Get the metadata of the boxed object.
    pub fn metadata(&self) -> <T as Pointee>::Metadata {
        self.metadata
    }

    /// Break a raw box into its component parts.
    pub fn into_raw_parts(self) -> (S::Handle, <T as Pointee>::Metadata, S) {
        let this = &*ManuallyDrop::new(self);
        (this.handle, this.metadata, unsafe {
            ptr::read(&this.storage)
        })
    }

    /// Reassemble a raw box from its component parts.
    pub unsafe fn from_raw_parts(
        handle: S::Handle,
        metadata: <T as Pointee>::Metadata,
        storage: S,
    ) -> Self {
        Self {
            handle,
            metadata,
            storage,
        }
    }
}

unsafe impl<#[may_dangle] T: ?Sized, S: Storage> Drop for RawBox<T, S> {
    fn drop(&mut self) {
        unsafe { self.storage.deallocate(self.handle, self.heap_layout()) }
    }
}

/// A pointer type for heap allocation. A tiny subset of std's Box.
pub struct Box<T: ?Sized, S: Storage> {
    raw: RawBox<T, S>,
}

impl<T: ?Sized, S: Storage> Box<T, S> {
    pub fn new_in(t: T, storage: S) -> Self
    where
        T: Sized,
    {
        let mut this = Self {
            raw: unsafe { RawBox::new((), storage) }
                .unwrap_or_else(|_| handle_alloc_error(Layout::new::<T>())),
        };
        unsafe { this.raw.as_mut_ptr().write(t) };
        this
    }

    pub fn into_raw_parts(this: Self) -> (S::Handle, <T as Pointee>::Metadata, S) {
        let this = ManuallyDrop::new(this);
        unsafe { ptr::read(&this.raw) }.into_raw_parts()
    }

    pub unsafe fn from_raw_parts(
        handle: S::Handle,
        metadata: <T as Pointee>::Metadata,
        storage: S,
    ) -> Self {
        Self {
            raw: RawBox::from_raw_parts(handle, metadata, storage),
        }
    }
}

impl<T: ?Sized, S: Storage> Deref for Box<T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw.as_ptr() }
    }
}

impl<T: ?Sized, S: Storage> DerefMut for Box<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw.as_mut_ptr() }
    }
}

unsafe impl<#[may_dangle] T: ?Sized, S: Storage> Drop for Box<T, S> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.raw.as_mut_ptr()) }
    }
}

unsafe impl<T, U: ?Sized, S: Storage> CoerciblePtr<U> for Box<T, S> {
    type Pointee = T;
    type Output = Box<U, S>;

    fn as_sized_ptr(&mut self) -> *mut Self::Pointee {
        self.raw.as_mut_ptr()
    }

    unsafe fn replace_ptr(self, ptr: *mut U) -> Self::Output {
        let (handle, (), storage) = Self::into_raw_parts(self);
        let (_, metadata) = ptr.to_raw_parts();
        Box::from_raw_parts(handle, metadata, storage)
    }
}
