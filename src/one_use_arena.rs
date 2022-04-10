use {
    crate::{Handle, Storage},
    core::{
        alloc::AllocError,
        cmp::Ordering,
        hash::{Hash, Hasher},
        mem::MaybeUninit,
        ptr::NonNull,
    },
};

pub struct OneUseArenaStorage<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    next: usize,
}

/// A handle into a [`ArenaStorage]`.
pub struct OneUseArenaStorageHandle {
    index: usize,
}

impl<T, const N: usize> OneUseArenaStorage<T, N> {
    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            next: 0,
        }
    }
}

unsafe impl<T, const N: usize> Storage<T> for OneUseArenaStorage<T, N> {
    type Handle = OneUseArenaStorageHandle;

    unsafe fn create(&mut self, (): ()) -> Result<Self::Handle, AllocError> {
        if self.next < N {
            let handle = OneUseArenaStorageHandle::new(self.next);
            self.next += 1;
            Ok(handle)
        } else {
            Err(AllocError)
        }
    }

    unsafe fn destroy(&mut self, _handle: Self::Handle) {}
    unsafe fn resolve_metadata(&self, _handle: Self::Handle) {}

    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<T> {
        NonNull::new_unchecked(self.data.get_unchecked(handle.index).as_ptr() as *mut T)
    }

    unsafe fn resolve_mut(&mut self, handle: Self::Handle) -> NonNull<T> {
        NonNull::new_unchecked(self.data.get_unchecked_mut(handle.index).as_mut_ptr())
    }
}

unsafe impl<T: ?Sized> Handle<T> for OneUseArenaStorageHandle {}

impl OneUseArenaStorageHandle {
    pub fn new(index: usize) -> Self {
        OneUseArenaStorageHandle { index }
    }
}

impl Copy for OneUseArenaStorageHandle {}
impl Clone for OneUseArenaStorageHandle {
    fn clone(&self) -> Self {
        *self
    }
}

impl Eq for OneUseArenaStorageHandle {}
impl PartialEq for OneUseArenaStorageHandle {
    fn eq(&self, _rhs: &Self) -> bool {
        true
    }
}

impl PartialOrd for OneUseArenaStorageHandle {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for OneUseArenaStorageHandle {
    fn cmp(&self, _rhs: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl Hash for OneUseArenaStorageHandle {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
