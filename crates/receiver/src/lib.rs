#![cfg_attr(not(feature = "std"), no_std)]

use xp_channel::Message;
use xp_format::{XbiFormat, XbiMetadata, XbiResult};

#[cfg(feature = "frame")]
pub mod frame;

/// The receiver part of the XBI channel. This simply receives a message and routes, depending on the implementation, to either the request or response handler.
///
/// Note: For asynchronous patterns, a receiver should have some way of sending the message back. A two-way channel is provided for this, however we should find inventive
/// ways of reasoning about these messages, and could follow a push based-pattern where the sender can check some storage item for it's response. This would require some
/// way of trustlessly verifying or perhaps just having roles who can update on behalf of the destination.
pub trait Receiver {
    /// The origin type for the channel
    type Origin;
    /// The expected outcome of the message
    type Outcome;

    /// Receive an XBI message
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
