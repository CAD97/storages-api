use {
    crate::{
        polyfill::{is_zst, Bool, True},
        AllocStorage, Box, InlineStorage, Storage,
    },
    core::{
        alloc::Allocator,
        marker::PhantomData,
        mem::{ManuallyDrop, MaybeUninit},
        ptr::{self, Pointee},
    },
};

/*

--------------------------------------------------------------------------------
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Read This! ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
--------------------------------------------------------------------------------


Okay, so sit down and brace yourself. This is quite complicated and subtle, and
basically *all* of it has to be understood in order to comprehend *any* of what
is going on here.

The goal is to be able to use RawBox<dyn Trait, DynStorage> as a polymorphic
storage that can be used to handle any of the following types uniformly:

- RawBox<T, AllocStorage<A>>
- RawBox<T, InlineStorage<usize>>
- RawBox<T, SmallStorage<usize>>
- &move T (equivalently &mut ManuallyDrop<T> where we claim the drop)
- T where Layout::new::<T>().fits_in(Layout::new::<usize>())
- *maybe* &mut T? figuring that out as I go...
- *maybe* other DerefMut smart pointers? figuring that out as I go...

and for this to all be the size of 2×usize -- that of Box<dyn T, Global>.

Naively, this would take 4×usize, and roughly look like

struct RawBox<dyn Trait, DynStorage<A>> {
    handle: NonNull<()>,
    metadata: &'static VTable<dyn <T as Trait>>,
    storage: struct RawBox {
        handle: (),
        metadata: &'static VTable<dyn Storage>,
        storage: SmallStorage<usize>,
    }
}

We can get this down to 3×usize fairly simply by using the SmallStorage trick:
switching between InlineStorage and AllocStorage (and just those two) based not
on a vtable, but on the size of the stored memory.

struct RawBox<dyn Trait, DynStorage<A>> {
    handle: (),
    metadata: &'static VTable<dyn <T as Trait>>,
    storage: SmallStorage<usize, A>,
}

but -- why don't we just use SmallStorage then? The reason, and problem, is that
we also want to support &move T, stealing the memory from somewhere else. This
allows handling types larger than usize without allocating memory.

Readers paying close attention will notice that the above raw box layout is
actually only 2×usize. That's why the simple DynStorage<A> results in a 3×usize
box: we use a handle of Option<NonNull<()>>, an a non-null pointer means that
we have a borrowed item, not a stored item.

struct RawBox<dyn Trait, DynStorage<'a, A>> {
    handle: Option<NonNull<()>>,
    metadata: &'static VTable<dyn <T as Trait>>,
    storage: DynStorage(SmallStorage<usize, A>),
}

If you want to support &mut T, note that &mut T behaves equivalently *here* to
&move ButDontDrop<T> (i.e. &move ManuallyDrop<T>). So long as T: Trait implies
ButDontDrop<T>: Trait, supporting &mut T is no extra work.[^1]

This is fairly trivial to implement, just requiring a bit of plumbing but no
real difficult implementation tricks. Note that we don't use Storage::allocate;
instead, we convert straight to RawBox<dyn Trait, DynStorage<A>>.

type Handle = DynHandle(Option<NonNull<()>>);
fn allocate, grow, shrink = unreachable!()
fn resolve[_mut] = if let Some(ptr) = handle {
    ptr::from_raw_parts[_mut](ptr, metadata)
} else {
    storage.resolve[_mut](())
}

Note that this implementation doesn't use the fact that we're restricting
ourselves to dyn Trait at all; it works for any type at all, with or without
unsizing. The utility of this specific solution doing extra state to support
inline storage probably isn't useful for e.g. slices, though.

The remaining redundancy in our 3×usize solution is that we either use the
handle *or* the storage, leaving a dead usize in our layout either way. How do
we remove this wasted space, and get a 2×usize layout? SmallStorage is valid for
any usize bit pattern, and NonNull<()> only has one invalid bit pattern, which
we're using to choose if we're borrowed memory or owned memory.

The solution is in the vtable. &move T and Box<T> only differ in one thing:
what dropping does. drop(&move T) does drop_in_place(*mut T). drop(Box<T>) does
drop_in_place(*mut T) plus dealloc(*mut T). Otherwise, the pointer is handled
exactly the same[^3].

This also means our DynStorage is no longer allocator specific, but it trades
this for another restriction: it must be only used with ZST allocators, as there
is no way to pass in an allocator parameter to the vtable dynamic drop_in_place.
The hope is that things like arena allocators would be able to make do with just
&move semantics and freeing the memory afterwards. This may need revisiting.

struct RawBox<dyn Trait, DynStorage<'a>> {
    handle: (),
    metadata: &'static VTable<dyn Trait'>,
    storage: DynStorage<'a>,
}

A simple implementation eagerly moves small types out of indirections and into
the inline storage. This means that it cannot be used for &mut T[^1], as it
would mutate the T in place for small Ts. The solution to this is to have &mut T
keep the indirection, but how do you tell apart small inline T and &mut T?

The answer is, again, to put the information into the vtable. It is [always safe
for an integer to pretend to be a pointer](ptr::invalid), so "all" that needs to
be done is to provide a vtable which turns the given pointer address back into
the small pointer-address-sized T and dispatches to Ts implementation. Simple
enough in theory, but complicated in practice, especially without compiler
support[^3].

It's much the same thing for other custom DerefMut smart pointers. So long as
the smart pointer can into_raw into *mut T and from_raw from *mut T, generating
a wrapping vtable is as simple as (whatever magic to generate such vtable and)
making drop_in_place from_raw the smart pointer to drop it.

Pointers which are *not* DerefMut are also theoretically suportable, so long as
we somehow provide two key guarantees:

- The trait only uses T by-reference, not by-mutable-reference, and
- It is considered unsound to downcast RawBox<dyn Trait, DynStorage<'a>> to a
  concrete type. Doing so is already *very* sketchy, though, given all of the
  magic going on to semi-transparently change the type (vtable) of the boxed
  object to handle the hyper-polymorphic drop_in_place.

--------------------------------------------------------------------------------
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Additional notes ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
--------------------------------------------------------------------------------

[^1]: &move ManuallyDrop<T> cannot be used for &mut T in general. The problem
(which the later scheme hits) is that &mut T requires any modification to
happen in-place, where &move deliberately makes *no* guarantees beyond leaving
the place in a valid-but-unspecified (ideally[^2] dropped) state.

[^2]: Remember: mem::forget is safe. &move T doesn't actually *have* to drop.

[^3]: In order to to provide wrapped vtables, we need compiler support. Instead,
we do something terrible: we just leak any heap allocation. Additionally, since
we can't wrap vtables, we move all small data inline, so that we don't need the
vtable to desmuggle pointer-sized values. We don't support &mut T; if T is large
enough to not fit inline, you can use &mut ManuallyDrop<ManuallyDrop<T>>, but we
acknowledge that this is a bad solution. What compiler support for this would
look like, though, I have no clue at the moment.

--------------------------------------------------------------------------------
~~~~~~~~~~~~~~~~~~~~~~~~~~~ Time for the Actual Impl ~~~~~~~~~~~~~~~~~~~~~~~~~~~
--------------------------------------------------------------------------------

... now that I've written 150 lines of justification 😅

This file is written in a semi literate code style, to best ensure that the code
and the justification match, and that everything works as expected. This code is
subtle -- even subtly subtle at that -- so deserves every chance it can get at
being more transparent to future readers.

*/

use core::{
    alloc::{AllocError, Layout},
    mem,
    ptr::DynMetadata,
};

use crate::{polyfill::layout_fits_in, Memory, SharedMutabilityStorage};

/// Dynamic single storage for use with `RawBox<dyn Trait, DynStorage<A>>`.
///
/// `DynStorage` cannot be constructed directly. Instead, you can convert any
/// of the following pointer types into `RawBox<dyn Trait, DynStorage>`, given
/// you have `T: Trait`:
///
/// - `RawBox<T, AllocStorage<A>>` where `size_of::<A>() == 0`
/// - `RawBox<T, InlineStorage<usize>>`
/// - `RawBox<T, SmallStorage<usize, A>>` where `size_of::<A>() == 0`
/// - `&mut ManuallyDrop<T>` (used as "`&move T`")
/// - `T` where `Layout::new::<T>().fits_in(Layout::new::<usize>())`
pub struct DynStorage<'a> {
    // In the storage, we store MaybeUninit<usize> as our actual data store. The
    // vtable of the boxed object is stored by the RawBox. This data store
    // stores one of two things:
    // - if Layout::new::<T>().fits_in(Layout::new::<usize>()), T
    // - else *mut T
    storage: MaybeUninit<usize>,
    // If we store a pointer, that pointer must not live its potentially
    // borrowed backing memory, so we note that we store a reference here.
    _marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Storage for DynStorage<'a> {
    // Our handle type is (); no extra data is stored in the raw box beyond the
    // storage itself and the pointer metadata. This ensures that our raw box
    // parts triple of (S::Handle, <dyn Trait as Pointee>::Metadata, S) is only
    // 2×usize big.
    type Handle = ();

    // Allocation cannot happen. There is no way to construct DynStorage
    // directly; it is only constructed as part of an already-allocated RawBox.
    // However, it can be acquired by RawBox::into_raw_parts, so we always fail
    // allocation, rather than panic or otherwise treat this as unreachable.
    fn allocate(&mut self, _: Layout) -> Result<Self::Handle, AllocError> {
        Err(AllocError)
    }

    /// Deallocation is a no-op. When the boxed T is dropped, the drop_in_place
    /// call handles any required deallocation.
    /// XXX: This might break the actual Box's normal API, as it isn't properly
    ///      "DerefPlace" anymore -- normally you can move out of a box and
    ///      dealloc it separately, or drop the contents of a box and then
    ///      reinitialize it with new contents. If comandeering drop_in_place<T>
    ///      like this isn't viable, we'll have to instead add a separate entry
    ///      for dealloc into the vtable at the end, so it can still be used as
    ///      the normal dyn Trait vtable. This might even be preferable if this
    ///      is done through more compiler magic than libs code.
    unsafe fn deallocate(&mut self, _: Self::Handle, _: Layout) {}

    unsafe fn resolve(&self, _: Self::Handle, layout: Layout) -> &Memory {
        if layout_fits_in(layout, Layout::for_value(&self.storage)) {
            // If the layout of the boxed object fits inline, it's inline. In a
            // full, vtable-wrapping implementation, we would return the object
            // typecast as a pointer, but the prototype inlines all small data.
            // XXX: resolve returns a reference currently. This is nice because
            //      the lifetime is obvious, rather than having to specify when
            //      a raw pointer is invalidated, but makes returning an invalid
            //      pointer definitely illegal...
            &*ptr::from_raw_parts(self.storage.as_ptr().cast(), layout.size())
        } else {
            // If it doesn't, then the inline data is a pointer to the object.
            &*ptr::from_raw_parts(
                self.storage.as_ptr().cast::<*const ()>().read(),
                layout.size(),
            )
        }
    }

    unsafe fn resolve_mut(&mut self, _: Self::Handle, layout: Layout) -> &mut Memory {
        if layout_fits_in(layout, Layout::for_value(&self.storage)) {
            // If the layout of the boxed object fits inline, it's inline. In a
            // full, vtable-wrapping implementation, we would return the object
            // typecast as a pointer, but the prototype inlines all small data.
            // XXX: resolve_mut returns a reference currently. This is nice
            //      because the lifetime is obvious, rather than having to
            //      specify when a raw pointer is invalidated, but makes
            //      returning an invalid pointer definitely illegal...
            &mut *ptr::from_raw_parts_mut(self.storage.as_mut_ptr().cast(), layout.size())
        } else {
            // If it doesn't, then the inline data is a pointer to the object.
            &mut *ptr::from_raw_parts_mut(
                self.storage.as_mut_ptr().cast::<*mut ()>().read(),
                layout.size(),
            )
        }
    }

    // Just like allocation, DynStorage does not support reallocation, as this
    // is not used by the RawBox API. Again as with allocate, though, these
    // methods could be called by splitting the box into its raw parts, so just
    // always fail; it is always safe behavior for reallocation to fail.
    // XXX: Consider making grow/shrink default implemented to always fail?
    unsafe fn grow(
        &mut self,
        _: Self::Handle,
        _: Layout,
        _: Layout,
    ) -> Result<Self::Handle, AllocError> {
        Err(AllocError)
    }

    unsafe fn shrink(
        &mut self,
        _: Self::Handle,
        _: Layout,
        _: Layout,
    ) -> Result<Self::Handle, AllocError> {
        Err(AllocError)
    }
}

// Now we come to the actual construction of dynamic storage boxes.
impl<'a, U> Box<U, DynStorage<'a>>
where
    // The unsized target type must be dyn Trait.
    U: ?Sized + Pointee<Metadata = DynMetadata<U>>,
{
    /// Construct a dynamic storage box from a standard box.
    pub fn boxed<A>(
        // We start with a heap-allocated object.
        boxed: Box<U, AllocStorage<A>>,
    ) -> Self
    where
        // The allocator must be trivial.
        A: Copy + Allocator,
        Bool<{ is_zst::<A>() }>: True,
    {
        // Do some paranoia checks that the alloc_storage is indeed trivial.
        assert_eq!(mem::size_of::<AllocStorage<A>>(), 0);
        assert!(!mem::needs_drop::<AllocStorage<A>>());

        // Get the layout of U before deconstructing the box.
        let layout = Layout::for_value::<U>(&*boxed);

        // Split the box into its alloc storage, vtable, and alloc handle.
        let (alloc_handle, vtable, alloc_storage) = Box::into_raw_parts(boxed);

        // Because we control AllocStorage, we know the handle is just a pointer
        // and that deallocating the handle is just calling Allocator::dealloc.
        // Convert the handle into just the pointer; forget the trivial storage.
        let ptr = unsafe { alloc_storage.resolve_raw(alloc_handle, layout) }.as_mut_ptr();
        #[allow(clippy::forget_non_drop)]
        mem::forget(alloc_storage);

        // This is where the vtable wrapping should happen, but this is not
        // currently possible without new compiler features, so we just let the
        // box leak instead, by using the vtable as-is.

        if layout_fits_in(layout, Layout::new::<usize>()) {
            // Because we don't do any vtable wrapping for this prototype, small
            // values have to be moved inline. Construct an inline box and call
            // the inline box conversion instead.
            let mut inline_storage = InlineStorage::<usize>::new();
            inline_storage.allocate(layout).unwrap(); // already checked layout fits
            unsafe {
                let inline_memory = inline_storage.resolve_mut((), layout);
                ptr::copy_nonoverlapping(ptr, inline_memory.as_mut_ptr(), layout.size());
                return Self::inline(Box::from_raw_parts((), vtable, inline_storage));
            }
        }

        // Construct the DynStorage holding the heap pointer.
        let mut dyn_storage = DynStorage {
            storage: MaybeUninit::uninit(),
            _marker: PhantomData,
        };
        unsafe {
            dyn_storage
                .storage
                .as_mut_ptr()
                .cast::<*mut ()>()
                .write(ptr.cast());
        }

        // Construct the sucessfully storage-erased box.
        unsafe { Box::from_raw_parts((), vtable, dyn_storage) }
    }

    /// Construct a dynamic storage box inline.
    pub fn inline(
        // We start with an inline-allocated object. Requiring boxing the value
        // externally simplifies things, but we *could* package it internally.
        boxed: Box<U, InlineStorage<usize>>,
    ) -> Self {
        // Split the box into its vtable and inline storage.
        let ((), vtable, inline_storage) = Box::into_raw_parts(boxed);

        // Because we control InlineStorage, we know it's just a transparent
        // wrapper around MaybeUninit<usize>. Rather than trying to handle U,
        // which is already unsized for us, we just move MaybeUninit<usize>
        // around, which we know contains the actual U.
        let memory = unsafe { mem::transmute::<_, MaybeUninit<usize>>(inline_storage) };

        // This is where the vtable wrapping should happen, but this is not
        // currently possible without new compiler features, so instead we've
        // ensured that small objects are always known to be inline.

        // Construct the DynStorage holding the heap pointer.
        let dyn_storage = DynStorage {
            storage: memory,
            _marker: PhantomData,
        };

        // Construct the sucessfully storage-erased box.
        unsafe { Box::from_raw_parts((), vtable, dyn_storage) }
    }

    /// Construct a dynamic storage box by taking someone else's allocation.
    pub unsafe fn take(
        // We start with a reference to ManuallyDrop which we claim to drop.
        taken: &'a mut ManuallyDrop<U>,
    ) -> Self {
        // Get the layout of U before deconstructing the reference.
        let layout = Layout::for_value::<U>(&*taken);

        // Split the reference into erased pointer and vtable.
        let (ptr, vtable) = (taken as *mut ManuallyDrop<U> as *mut U).to_raw_parts();

        if layout_fits_in(layout, Layout::new::<usize>()) {
            // Because we don't do any vtable wrapping for this prototype, small
            // values have to be moved inline. Construct an inline box and call
            // the inline box conversion instead.
            let mut inline_storage = InlineStorage::<usize>::new();
            inline_storage.allocate(layout).unwrap(); // already checked layout fits
            unsafe {
                let inline_memory = inline_storage.resolve_mut((), layout);
                ptr::copy_nonoverlapping(ptr.cast(), inline_memory.as_mut_ptr(), layout.size());
                return Self::inline(Box::from_raw_parts((), vtable, inline_storage));
            }
        }

        // Construct the DynStorage holding the borrowed pointer.
        let mut dyn_storage = DynStorage {
            storage: MaybeUninit::uninit(),
            _marker: PhantomData,
        };
        unsafe {
            dyn_storage
                .storage
                .as_mut_ptr()
                .cast::<*mut ()>()
                .write(ptr);
        }

        // Construct the sucessfully storage-erased box.
        unsafe { Box::from_raw_parts((), vtable, dyn_storage) }
    }
}
