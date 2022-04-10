//! A proof-of-concept implementation of (one version of) the storages proposal.
//!
//! Describing the raw storage API, we have:
//!
//! - [`Storage`]: a storage that can store objects
//! - [`SliceStorage`]: a storage for growable slices
//! - [`MultipleStorage`]: a storage that can store multiple objects
//! - [`SharedMutabilityStorage`] and [`PinningStorage`]
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
