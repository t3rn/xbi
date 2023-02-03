#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{sp_std, Either};
use sp_std::prelude::*;

pub use xbi_format::{XbiFormat, XbiMetadata, XbiResult};

#[cfg(feature = "frame")]
pub use xcm::{
    latest::{ExecuteXcm, SendXcm},
    prelude::*,
};

pub mod queue;
pub mod traits;

/// A message containing a request or a response
#[derive(Clone, Eq, PartialEq, Encode, Decode, TypeInfo, Debug)]
pub enum Message {
    Request(XbiFormat),
    Response(XbiResult, XbiMetadata),
}

impl Default for Message {
    fn default() -> Self {
        Self::Request(XbiFormat::default())
    }
}

impl From<XbiFormat> for Message {
    fn from(format: XbiFormat) -> Self {
        Message::Request(format)
    }
}

impl From<(XbiResult, XbiMetadata)> for Message {
    fn from(t: (XbiResult, XbiMetadata)) -> Self {
        Message::Response(t.0, t.1)
    }
}

impl Message {
    pub fn get_metadata(&self) -> &XbiMetadata {
        match self {
            Message::Request(x) => &x.metadata,
            Message::Response(_, x) => x,
        }
    }
}

/// A trait to allow emitting events or handling of events along the step of a message's lifecycle.
pub trait ChannelProgressionEmitter {
    /// This is emitted after sending the message over the transport protocol
    fn emit_sent(msg: Message);
    /// Emitted when the message is received on the destination
    fn emit_received(msg: Either<&XbiFormat, &XbiResult>);
    /// Emitted when the instruction for the message has been complete
    fn emit_instruction_handled(msg: &XbiFormat, weight: &u64);
    /// Emitted when the request is handled
    fn emit_request_handled(result: &XbiResult, metadata: &XbiMetadata, weight: &u64);
}

// Noop implementation
impl ChannelProgressionEmitter for () {
    fn emit_instruction_handled(_msg: &XbiFormat, _weight: &u64) {}

    fn emit_request_handled(_result: &XbiResult, _metadata: &XbiMetadata, _weight: &u64) {}

    fn emit_received(_msg: Either<&XbiFormat, &XbiResult>) {}

    fn emit_sent(_msg: Message) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Message;

    #[test]
    fn get_metadata() {
        let msg = Message::Request(Default::default());
        assert_eq!(msg.get_metadata(), &XbiMetadata::default());
        let msg = Message::Response(Default::default(), Default::default());
        assert_eq!(msg.get_metadata(), &XbiMetadata::default());
    }
}
