use {
    crate::{
        polyfill::{layout_fits_in, layout_for_meta},
        Handle, Storage,
    },
    core::{
        alloc::{AllocError, Layout},
        cmp::Ordering,
        mem::MaybeUninit,
        ptr::{NonNull, Pointee},
    },
};

// Note: I would like to have `<const LAYOUT: Layout>` instead, but
// feature(generic_const_exprs) is currently a little *too* incomplete
pub struct InlineStorage<DataStore> {
    data: MaybeUninit<DataStore>,
}

pub struct InlineStorageHandle<T: ?Sized> {
    meta: <T as Pointee>::Metadata,
}

impl<DataStore> InlineStorage<DataStore> {
    pub fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
        }
    }

    pub fn fits<T: ?Sized>(&self, meta: <T as core::ptr::Pointee>::Metadata) -> bool {
        let available_layout = Layout::new::<DataStore>();
        layout_for_meta::<T>(meta).map_or(false, |needed_layout| {
            layout_fits_in(needed_layout, available_layout)
        })
    }
}

unsafe impl<DataStore> Storage for InlineStorage<DataStore> {
    type Handle<T: ?Sized> = InlineStorageHandle<T>;

    unsafe fn create<T: ?Sized>(
        &mut self,
        meta: <T as core::ptr::Pointee>::Metadata,
    ) -> Result<Self::Handle<T>, AllocError> {
        let available_layout = Layout::new::<DataStore>();
        let needed_layout = layout_for_meta::<T>(meta).ok_or(AllocError)?;
        if layout_fits_in(needed_layout, available_layout) {
            Ok(InlineStorageHandle { meta })
        } else {
            Err(AllocError)
        }
    }

    unsafe fn destroy<T: ?Sized>(&mut self, _handle: Self::Handle<T>) {}

    unsafe fn resolve<T: ?Sized>(&self, handle: Self::Handle<T>) -> NonNull<T> {
        let ptr = NonNull::new_unchecked(self.data.as_ptr() as *mut ());
        NonNull::from_raw_parts(ptr.cast(), handle.meta)
    }

    unsafe fn resolve_mut<T: ?Sized>(&mut self, handle: Self::Handle<T>) -> NonNull<T> {
        let ptr = NonNull::new_unchecked(self.data.as_mut_ptr() as *mut ());
        NonNull::from_raw_parts(ptr, handle.meta)
    }
}

unsafe impl<T: ?Sized> Handle<T> for InlineStorageHandle<T> {
    fn metadata(self) -> <T as Pointee>::Metadata {
        self.meta
    }
}

impl<T: ?Sized> InlineStorageHandle<T> {
    pub fn new(meta: <T as Pointee>::Metadata) -> Self {
        InlineStorageHandle { meta }
    }
}

impl<T: ?Sized> Copy for InlineStorageHandle<T> {}
impl<T: ?Sized> Clone for InlineStorageHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Eq for InlineStorageHandle<T> {}
impl<T: ?Sized> PartialEq for InlineStorageHandle<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.meta == rhs.meta
    }
}

impl<T: ?Sized> PartialOrd for InlineStorageHandle<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl<T: ?Sized> Ord for InlineStorageHandle<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.meta.cmp(&rhs.meta)
    }
}
