/// provides some trait for storage
///
///
/// could use sp-trie
///
///is injected into the module that uses it, minimally sender, possible receiver

// TODO: needs to be 100% consistent accross nodes

// substrate-trie = { path = "../trie", default-features = false }
// trie-db = { version = "0.12", default-features = false }
// memory-db = { version = "0.12", default-features = false }
// use primitives::Blake2Hasher;
// use trie_db::{TrieMut, Trie};
// use substrate_trie::{TrieDB, TrieDBMut, PrefixedMemoryDB};
// fn bla() {
//     let pairs = [
//         (b"0103000000000000000464".to_vec(), b"0400000000".to_vec()),
//         (b"0103000000000000000469".to_vec(), b"0401000000".to_vec()),
//     ].to_vec();
//
//     let mut mdb = PrefixedMemoryDB::default();
//     let mut root = rstd::default::Default::default();
//     let _ = {
//         let v = &pairs;
//         let mut t = TrieDBMut::<Blake2Hasher>::new(&mut mdb, &mut root);
//         for i in 0..v.len() {
//             let key: &[u8]= &v[i].0;
//             let val: &[u8] = &v[i].1;
//             t.insert(key, val).expect("static input");
//         }
//         t
//     };
//
//     let trie = TrieDB::<Blake2Hasher>::new(&mdb, &root).expect("on memory with static content");
//
//     let iter = trie.iter().expect("static input");
//     let mut iter_pairs = Vec::new();
//     for pair in iter {
//         let (key, value) = pair.expect("on memory with static content");
//         iter_pairs.push((key, value.to_vec()));
//     }
//     iter_pairs.len() as u64
// }

// FIXME: lots of duplicated storage, storing the XbiCheckin everywhere, store it once and then map the hash for pending/queued
// store hashes into pending, queued, no need to update state

// /// Queue XBI for batch execution
// #[pallet::storage]
// pub type XBICheckInsQueued<T> = StorageValue<
//     _,
//     BoundedVec<<T as frame_system::Config>::Hash>,
//     ValueQuery,
// >;
//
// /// Processed XBI queue pending for execution
// #[pallet::storage]
// pub type XBICheckInsPending<T> = StorageValue<
//     _,
//     BoundedVec<<T as frame_system::Config>::Hash>,
//     ValueQuery,
// >;
//
// #[pallet::storage]
// /// XBI called for execution
// pub type XBICheckIns<T> = StorageMap<
//     _,
//     Identity,
//     <T as frame_system::Config>::Hash,
//     XBICheckIn<<T as frame_system::Config>::BlockNumber>,
//     OptionQuery,
// >;
//
// /// Lifecycle: If executed: XBICheckInsPending -> XBICheckIns -> XBICheckOutsQueued
// /// Lifecycle: If not executed: XBICheckInsPending -> XBICheckOutsQueued
// pub type XBICheckOutsQueued<T> =
// StorageValue<_, Identity, BoundedVec<<T as frame_system::Config>::Hash>;
//
// pub type XBICheckOuts<T> =
// StorageMap<_, Identity, <T as frame_system::Config>::Hash, XBICheckOut, OptionQuery>;
pub enum QueueKind {
    Checkin,
    Checkout,
}
/// XbiStore needs two queues, one for checkins and one for checkouts.
///
/// When the sender sends a message:
///     A checkin is inserted, this holds the xbi message
///     The sender would then queue the message to the queue
///     The sender would post
///
///
/// The sender will post a checkin to the queue when it sends a message.
/// It will also post a checkout to the queue ready to receive a message.
pub trait XbiStore<Key> {
    type Error;

    /// Push an XBI format into the queue, pushing to the queued vector, and storing the XbiCheckin
    fn enqueue(val: XBIFormat) -> Result<(), Self::Error>;
    /// Remove an XBI format from queue
    fn dequeue(key: Key) -> Result<XBIFormat, Self::Error>;
    /// Promote a queued item to the pending queue, queue a pending checkout to the checkout queue
    fn pending(key: Key) -> Result<(), Self::Error>;

    /// Remove a pending checkout and push the result to the checkouts
    fn commit<Val>(key: Key, val: Val) -> Result<(), Self::Error>;
}
