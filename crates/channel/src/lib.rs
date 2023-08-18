#![feature(result_option_inspect)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod receiver;
pub mod sender;

pub use receiver::Receiver;
pub use sender::Sender;

pub fn xbi_origin<T: codec::Decode>(
    m: &xp_channel::XbiMetadata,
) -> Result<T, sp_runtime::DispatchError> {
    let x: Result<T, sp_runtime::DispatchError> = m
        .get_origin()
        .ok_or_else(|| "XBI message has no origin".into())
        .and_then(|o| {
            codec::Decode::decode(&mut o.as_ref())
                .map_err(|_| "XBI message origin is not valid".into())
        });
    x
}
