# storages-api

A proof-of-concept implementation of (one version of) the storages proposal.

Describing the raw storage API, we have:
- [`Storage`]: a storage that can store objects
- [`SliceStorage`]: a storage for growable slices
- [`MultipleStorage`]: a storage that can store multiple objects
- [`SharedMutabilityStorage`] and [`PinningStorage`]

Providing a safe wrapper around `Storage` use (up to uninit memory):
- [`RawBox`]: a raw (uninit payload) version of std `Box`
- [`RawVec`]: a raw (uninit payload) version of std `Vec`

Useful implementations of [`Storage`]:
- [`InlineStorage`]: single storage located in the storage's bytes
- [`AllocStorage`]: full-featured storage via allocation
- [`SmallStorage`]: inline storage with a fallback to allocation

[`Storage`]: https://cad97.github.io/storages-api/storage_api/trait.Storage.html
[`SliceStorage`]: https://cad97.github.io/storages-api/storage_api/trait.SliceStorage.html
[`MultipleStorage`]: https://cad97.github.io/storages-api/storage_api/trait.MultipleStorage.html
[`SharedMutabilityStorage`]: https://cad97.github.io/storages-api/storage_api/trait.SharedMutabilityStorage.html
[`PinningStorage`]: https://cad97.github.io/storages-api/storage_api/trait.PinningStorage.html
[`RawBox`]: https://cad97.github.io/storages-api/storage_api/struct.RawBox.html
[`RawVec`]: https://cad97.github.io/storages-api/storage_api/struct.RawVec.html
[`InlineStorage`]: https://cad97.github.io/storages-api/storage_api/struct.InlineStorage.html
[`AllocStorage`]: https://cad97.github.io/storages-api/storage_api/struct.AllocStorage.html
[`SmallStorage`]: https://cad97.github.io/storages-api/storage_api/struct.SmallStorage.html
