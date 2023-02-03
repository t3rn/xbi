use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchErrorWithPostInfo, weights::Weight};
use sp_std::prelude::*;
use xbi_format::XbiFormat;

/// A set of traits containing some loosely typed shims to storage interactions in substrate.
///
/// NOTE: the shims in this module are largely so that we can have a slim interface for some genric queues
/// without relying on `frame`. Ideally, these structures would be extracted out of frame in the future.
/// But for now we limit to what we have.
///
/// Due to the above limitations, some functionality is omitted, since it isn't required for our use case.
pub mod shims;

/// A structure containing the output of an instruction handle. This should be used to hold any results and error information.
/// Which might be relevant to the user.
///
/// This also adds information about weight used by the instruction handler.
#[derive(Encode, Decode, Default)]
pub struct HandlerInfo {
    // TODO[Optimisation]: We can bound the size, but ideally this should be configured by the user who sends the message.
    // We have ideas on how to specify this in future releases.
    pub output: Vec<u8>,
    // The weight that was used to handle the message.
    pub weight: Weight,
}

impl From<Weight> for HandlerInfo {
    fn from(i: Weight) -> Self {
        HandlerInfo {
            weight: i,
            ..Default::default()
        }
    }
}

impl From<Vec<u8>> for HandlerInfo {
    fn from(bytes: Vec<u8>) -> Self {
        HandlerInfo {
            output: bytes,
            ..Default::default()
        }
    }
}

impl From<(Vec<u8>, Weight)> for HandlerInfo {
    fn from(t: (Vec<u8>, Weight)) -> Self {
        let (bytes, i) = t;
        HandlerInfo {
            output: bytes,
            weight: i,
        }
    }
}

pub trait XbiInstructionHandler<Origin> {
    fn handle(
        origin: &Origin,
        xbi: &mut XbiFormat,
    ) -> Result<HandlerInfo, DispatchErrorWithPostInfo>;
}
