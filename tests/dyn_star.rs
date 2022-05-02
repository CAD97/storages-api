#![feature(allocator_api, generic_const_exprs)]
#![allow(incomplete_features)]

use std::mem::ManuallyDrop;

extern crate std;

use {
    std::{alloc::Global, fmt::Debug},
    storage_api::{AllocStorage, Box, DynStorage, InlineStorage},
    unsize::*,
};

fn debug_print(x: Box<dyn Debug, DynStorage<'_>>) {
    dbg!(&*x);
}

const GLOBAL: AllocStorage<Global> = AllocStorage::new(Global);
const INLINE: InlineStorage<usize> = InlineStorage::new();

#[test]
#[cfg_attr(all(miri, not(miri_ignore_leaks)), ignore = "leaks")]
fn heap_allocated() {
    let string: Box<String, _> = Box::new_in(String::from("Hello, world!"), GLOBAL);
    let string: Box<dyn Debug, _> = string.unsize(Coercion::to_debug());
    debug_print(Box::boxed(string));
    // Note: the proof of concept leaks here, since we can't wrap
    // vtables to tell apart owned versus borrowed pointers :(
}

#[test]
fn inline_allocated() {
    let number: Box<u16, _> = Box::new_in(42, INLINE);
    let number: Box<dyn Debug, _> = number.unsize(Coercion::to_debug());
    debug_print(Box::inline(number));
}

#[test]
fn stolen_allocation() {
    let mut string = ManuallyDrop::new(String::from("Hello, world!"));
    debug_print(unsafe { Box::take(&mut string) });
    // Note: no leak here, as the box takes and drops the ManuallyDrop contents.
}
