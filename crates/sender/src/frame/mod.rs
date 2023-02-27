use crate::Vec;
use xp_channel::Message;

pub mod queue_backed;
pub mod sync;

/// A trait requiring some implementer to provide a way for us to transform some type T
/// into a dispatchable call to receiver
/// The implementer should take note that this can be very different depending on who implements the channel.
/// Such as the indexes for the receiver, perhaps it should be indexed.
///
/// The implementer should implement this trait since it cannot be known by this crate, e.g:
///
/// ```
/// # use std::collections::VecDeque;
/// # pub fn test<T: frame_system::Config>(format: xp_channel::XbiFormat) {
///     //let pallet_index_in_runtime = 200;
///     //let mut xbi_call: VecDeque<u8> =
///     //   crate::pallet::Call::receive::<T> { msg: format.into() }
///     //       .encode()
///     //       .into();
///     //xbi_call.push_front(pallet_index_in_runtime); // Pallet index is not known by the crate for every channel and can be changed anytime
/// # }
/// ```
pub trait ReceiveCallProvider {
    fn provide<T: Into<Message>>(t: T) -> Vec<u8>;
}
