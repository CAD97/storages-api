//! A proof-of-concept implementation of (one version of) the storages proposal.
//!
//! Describing the raw storage API, we have:
//!
//! - [`Storage`]: a storage that manages a single memory allocation
//! - [`MultipleStorage`]: a storage that can manage multiple handles
//! - [`SharedMutabilityStorage`] and [`PinningStorage`] for advanced use
//!
//! Providing a safe wrapper around `Storage` use (up to uninit memory):
//!
//! - [`RawBox`]: a raw (uninit payload) version of std `Box`
//! - [`RawVec`]: a raw (uninit payload) version of std `Vec`
//!
//! Useful implementations of [`Storage`]:
//!
//! - [`InlineStorage`]: single storage located in the storage's bytes
//! - [`AllocStorage`]: full-featured storage via allocation
//! - [`SmallStorage`]: inline storage with a fallback to allocation
//! - [`BorrowedStorage`]: single storage located in someone else's memory

#![no_std]
#![feature(
    allocator_api,
    dropck_eyepatch,
    extern_types,
    layout_for_ptr,
    let_chains,
    maybe_uninit_array_assume_init,
    specialization,
    ptr_metadata
)]
#![allow(
    clippy::len_without_is_empty,
    clippy::missing_safety_doc,
    clippy::mut_from_ref,
    clippy::new_without_default,
    clippy::should_implement_trait,
    incomplete_features,
    unused_unsafe
)]

mod alloc;
mod borrowed;
mod inline;
mod polyfill;
mod raw_box;
mod raw_vec;
mod small;
mod traits;

#[doc(inline)]
pub use crate::{
    alloc::{AllocHandle, AllocStorage},
    borrowed::BorrowedStorage,
    inline::InlineStorage,
    raw_box::RawBox,
    raw_vec::RawVec,
    small::SmallStorage,
    traits::{Memory, MultipleStorage, PinningStorage, SharedMutabilityStorage, Storage},
};
