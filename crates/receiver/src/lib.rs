#![cfg_attr(not(feature = "std"), no_std)]

use xbi_channel_primitives::Message;
use xbi_format::{XbiFormat, XbiMetadata, XbiResult};

#[cfg(feature = "frame")]
pub mod frame;

pub trait Receiver {
    type Origin;
    type Outcome;

    fn receive(origin: Self::Origin, msg: Message) -> Self::Outcome {
        match msg {
            Message::Request(mut msg) => Self::handle_request(&origin, &mut msg),
            Message::Response(msg, metadata) => Self::handle_response(&origin, &msg, &metadata),
        }
    }

    /// Request should always run the instruction, and product some dispatch info containing meters for the execution
    fn handle_request(origin: &Self::Origin, msg: &mut XbiFormat) -> Self::Outcome;

    /// Response should update the state of the storage checkout queues and notify the sender of completion
    fn handle_response(
        caller: &Self::Origin,
        msg: &XbiResult,
        metadata: &XbiMetadata,
    ) -> Self::Outcome;
}
