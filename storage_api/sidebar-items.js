initSidebarItems({"struct":[["AllocStorage","A storage that stores objects via an [`Allocator`]."],["InlineStorage","A single storage which stores objects inline."],["InlineStorageHandle","A handle into an [`InlineStorage]`."],["RawBox","A raw box around some storage. Bundles the storage and its handle."],["RawVec","A raw vec around some slice storage. Bundles the storage and its handle."],["RefStorage","A storage wrapper around `&mut T`."],["RefStorageHandle","A handle into a [`RefStorage]`."],["SmallStorage","A single storage which stores objects inline if it fits, otherwise falling back to using an [`Allocator`]."],["SmallStorageHandle",""]],"trait":[["Handle","Types which can be used as a storage handle."],["MultipleStorage","A storage that can create multiple handles."],["PinningStorage","A storage that creates pinned handles."],["SharedMutabilityStorage","A storage that serves as a uniqueness barrier."],["SliceStorage","A storage that can reallocate to adjust the length of slice objects."],["Storage","Types which can be used to store objects."]]});