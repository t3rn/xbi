use codec::{Decode, EncodeLike, FullCodec, FullEncode};
use sp_std::prelude::*;

/// A trait for working with macro-generated storage values under the substrate storage API.
///
/// Details on implementation can be found at [`generator::StorageValue`].
pub trait StorageValue<T: FullCodec> {
    /// The type that get/take return.
    type Query;

    /// Get the storage key.
    fn hashed_key() -> [u8; 32];

    /// Does the value (explicitly) exist in storage?
    fn exists() -> bool;

    /// Load the value from the provided storage instance.
    fn get() -> Self::Query;

    // Justification: not our trait
    #[allow(clippy::result_unit_err)]
    /// Try to get the underlying value from the provided storage instance.
    ///
    /// Returns `Ok` if it exists, `Err` if not.
    fn try_get() -> Result<T, ()>;

    // Justification: not our trait
    #[allow(clippy::result_unit_err)]
    /// Translate a value from some previous type (`O`) to the current type.
    ///
    /// `f: F` is the translation function.
    ///
    /// Returns `Err` if the storage item could not be interpreted as the old type, and Ok, along
    /// with the new value if it could.
    ///
    /// NOTE: This operates from and to `Option<_>` types; no effort is made to respect the default
    /// value of the original type.
    ///
    /// # Warning
    ///
    /// This function must be used with care, before being updated the storage still contains the
    /// old type, thus other calls (such as `get`) will fail at decoding it.
    ///
    /// # Usage
    ///
    /// This would typically be called inside the module implementation of on_runtime_upgrade, while
    /// ensuring **no usage of this storage are made before the call to `on_runtime_upgrade`**.
    /// (More precisely prior initialized modules doesn't make use of this storage).
    fn translate<O: Decode, F: FnOnce(Option<O>) -> Option<T>>(f: F) -> Result<Option<T>, ()>;

    /// Store a value under this key into the provided storage instance.
    fn put<Arg: EncodeLike<T>>(val: Arg);

    /// Store a value under this key into the provided storage instance; this uses the query
    /// type rather than the underlying value.
    fn set(val: Self::Query);

    /// Mutate the value
    fn mutate<R, F: FnOnce(&mut Self::Query) -> R>(f: F) -> R;

    /// Mutate the value if closure returns `Ok`
    fn try_mutate<R, E, F: FnOnce(&mut Self::Query) -> Result<R, E>>(f: F) -> Result<R, E>;

    /// Clear the storage value.
    fn kill();

    /// Take a value from storage, removing it afterwards.
    fn take() -> Self::Query;
}

#[cfg(feature = "frame")]
impl<T: FullCodec, G: frame_support::StorageValue<T>> StorageValue<T> for G {
    type Query = G::Query;

    fn hashed_key() -> [u8; 32] {
        G::hashed_key()
    }

    fn exists() -> bool {
        G::exists()
    }

    fn get() -> Self::Query {
        G::get()
    }

    fn try_get() -> Result<T, ()> {
        G::try_get()
    }

    fn translate<O: Decode, F: FnOnce(Option<O>) -> Option<T>>(f: F) -> Result<Option<T>, ()> {
        G::translate(f)
    }

    fn put<Arg: EncodeLike<T>>(val: Arg) {
        G::put(val)
    }

    fn set(val: Self::Query) {
        G::set(val)
    }

    fn kill() {
        G::kill()
    }

    fn mutate<R, F: FnOnce(&mut Self::Query) -> R>(f: F) -> R {
        G::mutate(f)
    }

    fn try_mutate<R, E, F: FnOnce(&mut Self::Query) -> Result<R, E>>(f: F) -> Result<R, E> {
        G::try_mutate(f)
    }

    fn take() -> Self::Query {
        G::take()
    }
}

/// A strongly-typed map in storage.
///
/// Details on implementation can be found at [`generator::StorageMap`].
pub trait StorageMap<K: FullEncode, V: FullCodec> {
    /// The type that get/take return.
    type Query;

    /// Get the storage key used to fetch a value corresponding to a specific key.
    fn hashed_key_for<KeyArg: EncodeLike<K>>(key: KeyArg) -> Vec<u8>;

    /// Does the value (explicitly) exist in storage?
    fn contains_key<KeyArg: EncodeLike<K>>(key: KeyArg) -> bool;

    /// Load the value associated with the given key from the map.
    fn get<KeyArg: EncodeLike<K>>(key: KeyArg) -> Self::Query;

    /// Store or remove the value to be associated with `key` so that `get` returns the `query`.
    fn set<KeyArg: EncodeLike<K>>(key: KeyArg, query: Self::Query);

    // Justification: not our trait
    #[allow(clippy::result_unit_err)]
    /// Try to get the value for the given key from the map.
    ///
    /// Returns `Ok` if it exists, `Err` if not.
    fn try_get<KeyArg: EncodeLike<K>>(key: KeyArg) -> Result<V, ()>;

    /// Swap the values of two keys.
    fn swap<KeyArg1: EncodeLike<K>, KeyArg2: EncodeLike<K>>(key1: KeyArg1, key2: KeyArg2);

    /// Store a value to be associated with the given key from the map.
    fn insert<KeyArg: EncodeLike<K>, ValArg: EncodeLike<V>>(key: KeyArg, val: ValArg);

    /// Remove the value under a key.
    fn remove<KeyArg: EncodeLike<K>>(key: KeyArg);

    /// Mutate the value under a key.
    fn mutate<KeyArg: EncodeLike<K>, R, F: FnOnce(&mut Self::Query) -> R>(key: KeyArg, f: F) -> R;

    /// Mutate the item, only if an `Ok` value is returned.
    fn try_mutate<KeyArg: EncodeLike<K>, R, E, F: FnOnce(&mut Self::Query) -> Result<R, E>>(
        key: KeyArg,
        f: F,
    ) -> Result<R, E>;

    /// Mutate the value under a key.
    ///
    /// Deletes the item if mutated to a `None`.
    fn mutate_exists<KeyArg: EncodeLike<K>, R, F: FnOnce(&mut Option<V>) -> R>(
        key: KeyArg,
        f: F,
    ) -> R;

    /// Mutate the item, only if an `Ok` value is returned. Deletes the item if mutated to a `None`.
    /// `f` will always be called with an option representing if the storage item exists (`Some<V>`)
    /// or if the storage item does not exist (`None`), independent of the `QueryType`.
    fn try_mutate_exists<KeyArg: EncodeLike<K>, R, E, F: FnOnce(&mut Option<V>) -> Result<R, E>>(
        key: KeyArg,
        f: F,
    ) -> Result<R, E>;

    /// Take the value under a key.
    fn take<KeyArg: EncodeLike<K>>(key: KeyArg) -> Self::Query;
}

#[cfg(feature = "frame")]
impl<K: FullCodec, V: FullCodec, G: frame_support::StorageMap<K, V>> StorageMap<K, V> for G {
    type Query = G::Query;

    fn hashed_key_for<KeyArg: EncodeLike<K>>(key: KeyArg) -> Vec<u8> {
        G::hashed_key_for(key)
    }

    fn swap<KeyArg1: EncodeLike<K>, KeyArg2: EncodeLike<K>>(key1: KeyArg1, key2: KeyArg2) {
        G::swap(key1, key2)
    }

    fn contains_key<KeyArg: EncodeLike<K>>(key: KeyArg) -> bool {
        G::contains_key(key)
    }

    fn get<KeyArg: EncodeLike<K>>(key: KeyArg) -> Self::Query {
        G::get(key)
    }

    fn try_get<KeyArg: EncodeLike<K>>(key: KeyArg) -> Result<V, ()> {
        G::try_get(key)
    }

    fn set<KeyArg: EncodeLike<K>>(key: KeyArg, query: Self::Query) {
        G::set(key, query)
    }

    fn insert<KeyArg: EncodeLike<K>, ValArg: EncodeLike<V>>(key: KeyArg, val: ValArg) {
        G::insert(key, val)
    }

    fn remove<KeyArg: EncodeLike<K>>(key: KeyArg) {
        G::remove(key)
    }

    fn mutate<KeyArg: EncodeLike<K>, R, F: FnOnce(&mut Self::Query) -> R>(key: KeyArg, f: F) -> R {
        G::mutate(key, f)
    }

    fn mutate_exists<KeyArg: EncodeLike<K>, R, F: FnOnce(&mut Option<V>) -> R>(
        key: KeyArg,
        f: F,
    ) -> R {
        G::mutate_exists(key, f)
    }

    fn try_mutate<KeyArg: EncodeLike<K>, R, E, F: FnOnce(&mut Self::Query) -> Result<R, E>>(
        key: KeyArg,
        f: F,
    ) -> Result<R, E> {
        G::try_mutate(key, f)
    }

    fn try_mutate_exists<KeyArg: EncodeLike<K>, R, E, F: FnOnce(&mut Option<V>) -> Result<R, E>>(
        key: KeyArg,
        f: F,
    ) -> Result<R, E> {
        G::try_mutate_exists(key, f)
    }

    fn take<KeyArg: EncodeLike<K>>(key: KeyArg) -> Self::Query {
        G::take(key)
    }
}
