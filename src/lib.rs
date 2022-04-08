#![no_std]
#![feature(
    allocator_api,
    dropck_eyepatch,
    generic_associated_types,
    layout_for_ptr,
    specialization,
    ptr_metadata
)]
#![allow(
    clippy::len_without_is_empty,
    clippy::missing_safety_doc,
    clippy::new_without_default,
    incomplete_features,
    unused_unsafe
)]

mod alloc;
mod inline;
mod polyfill;
mod raw_box;
mod raw_vec;
mod small;
mod traits;

#[doc(inline)]
pub use crate::{
    alloc::AllocStorage,
    inline::{InlineStorage, InlineStorageHandle},
    raw_box::RawBox,
    raw_vec::RawVec,
    small::{SmallStorage, SmallStorageHandle},
    traits::{
        Handle, MultipleStorage, PinningStorage, SharedMutabilityStorage, SliceStorage, Storage,
    },
};
