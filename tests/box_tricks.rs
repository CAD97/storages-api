#![feature(allocator_api, ptr_metadata)]
#![allow(clippy::missing_safety_doc)]

use {
    cool_asserts::assert_panics,
    std::{
        alloc::{handle_alloc_error, Global, Layout},
        mem::{size_of, size_of_val, ManuallyDrop},
        panic::UnwindSafe,
    },
    storage_api::{AllocStorage, RawBox, RefStorage, RefStorageHandle, Storage},
};

pub struct Box<T: ?Sized, S: Storage<T> = AllocStorage<Global>> {
    raw: RawBox<T, S>,
}

impl<T> Box<T> {
    pub fn new(it: T) -> Self {
        unsafe {
            let mut raw = RawBox::<T, _>::new((), AllocStorage::new(Global))
                .unwrap_or_else(|_| handle_alloc_error(Layout::new::<T>()));
            raw.as_mut_ptr().write(it);
            Box { raw }
        }
    }
}

impl<T: ?Sized, S: Storage<T>> Drop for Box<T, S> {
    fn drop(&mut self) {
        unsafe { self.raw.as_mut_ptr().drop_in_place() }
    }
}

type MoveRef<'a, T> = Box<T, RefStorage<'a, ManuallyDrop<T>>>;

impl<'a, T: ?Sized> MoveRef<'a, T> {
    pub unsafe fn new_unchecked(it: &'a mut ManuallyDrop<T>) -> Self {
        Self {
            raw: RawBox::from_raw_parts(RefStorageHandle, RefStorage::new(it)),
        }
    }
}

impl<T: ?Sized, S: Storage<T>> UnwindSafe for Box<T, S> {}

#[test]
fn move_ref() {
    struct PanicDrop {
        message: &'static str,
    }
    impl Drop for PanicDrop {
        fn drop(&mut self) {
            panic!("{}", self.message)
        }
    }
    let mut place = ManuallyDrop::new(PanicDrop { message: "dropped" });
    let my_ref = unsafe { MoveRef::new_unchecked(&mut place) };
    assert_eq!(size_of_val(&my_ref), size_of::<&mut PanicDrop>());
    assert_panics!(drop(my_ref), includes("dropped"));
}
