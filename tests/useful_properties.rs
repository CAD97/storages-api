#![feature(allocator_api)]

extern crate std;

use {
    std::{
        alloc::{AllocError, Allocator, Global},
        mem::size_of,
        prelude::rust_2021::*,
    },
    storage_api::{RawBox, SmallStorage},
};

trait Trait {}

struct NullAlloc;

unsafe impl Allocator for NullAlloc {
    fn allocate(&self, _: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    unsafe fn deallocate(&self, _: std::ptr::NonNull<u8>, _: std::alloc::Layout) {
        unreachable!()
    }
}

type SmallRawBox<T, A> = RawBox<T, SmallStorage<usize, A>>;

#[test]
fn sizes() {
    assert_eq!(
        size_of::<SmallRawBox<dyn Trait, Global>>(),
        size_of::<Box<dyn Trait>>()
    );
}

#[test]
fn does_not_alloc() {
    let storage = SmallStorage::new(NullAlloc);
    unsafe {
        let mut boxed = SmallRawBox::new((), storage).unwrap_or_else(|_| panic!());
        *boxed.as_mut_ptr() = 0usize;
    }
}
