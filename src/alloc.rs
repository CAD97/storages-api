use {
    crate::{Memory, MultipleStorage, PinningStorage, SharedMutabilityStorage, Storage},
    core::{
        alloc::{AllocError, Allocator, Layout},
        mem::MaybeUninit,
        ptr::NonNull,
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

unsafe impl<A: Allocator> Storage for AllocStorage<A> {
    type Handle = AllocHandle;

    fn allocate(&mut self, layout: Layout) -> Result<Self::Handle, AllocError> {
        let (ptr, _meta) = self.alloc.allocate(layout)?.to_raw_parts();
        Ok(AllocHandle::new(ptr))
    }

    unsafe fn deallocate(&mut self, handle: Self::Handle, layout: Layout) {
        self.alloc.deallocate(handle.pointer.cast(), layout)
    }

    unsafe fn resolve(&self, handle: Self::Handle, layout: Layout) -> &Memory {
        NonNull::from_raw_parts(handle.pointer, layout.size()).as_ref()
    }

    unsafe fn resolve_mut(&mut self, handle: Self::Handle, layout: Layout) -> &mut Memory {
        NonNull::from_raw_parts(handle.pointer, layout.size()).as_mut()
    }

    unsafe fn grow(
        &mut self,
        handle: Self::Handle,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError> {
        let (ptr, _meta) = self
            .alloc
            .grow(handle.pointer.cast(), old_layout, new_layout)?
            .to_raw_parts();
        Ok(AllocHandle::new(ptr))
    }

    unsafe fn shrink(
        &mut self,
        handle: Self::Handle,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<Self::Handle, AllocError> {
        let (ptr, _meta) = self
            .alloc
            .shrink(handle.pointer.cast(), old_layout, new_layout)?
            .to_raw_parts();
        Ok(AllocHandle::new(ptr))
    }
}

unsafe impl<A: Allocator> MultipleStorage for AllocStorage<A> {
    unsafe fn resolve_many_mut<const N: usize>(
        &mut self,
        handles: [(Self::Handle, Layout); N],
    ) -> [&mut Memory; N] {
        let mut ptrs: [MaybeUninit<&mut Memory>; N] = MaybeUninit::uninit().assume_init();
        for (ptr, (handle, layout)) in ptrs.iter_mut().zip(handles) {
            ptr.write(self.resolve_raw(handle, layout));
        }
        MaybeUninit::array_assume_init(ptrs)
    }
}

unsafe impl<A: Allocator> SharedMutabilityStorage for AllocStorage<A> {
    unsafe fn resolve_raw(&self, handle: Self::Handle, layout: Layout) -> &mut Memory {
        NonNull::from_raw_parts(handle.pointer, layout.size()).as_mut()
    }
}

unsafe impl<A: Allocator> PinningStorage for AllocStorage<A> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AllocHandle {
    pointer: NonNull<()>,
}

unsafe impl Send for AllocHandle {}
unsafe impl Sync for AllocHandle {}

impl AllocHandle {
    const fn new<T: ?Sized>(pointer: NonNull<T>) -> Self {
        Self {
            pointer: pointer.cast(),
        }
    }
}
