use codec::Encode;
use frame_support::{
    dispatch::DispatchErrorWithPostInfo, pallet_prelude::DispatchResultWithPostInfo,
    weights::PostDispatchInfo,
};
use sp_std::prelude::*;
use xbi_channel_primitives::{traits::HandlerInfo, ChannelProgressionEmitter};
use xbi_format::{Status, XbiFormat, XbiMetadata, XbiResult};

pub mod queue_backed;
pub mod sync;

/// The receiver needs to invert the message src/dest so that it can respond accordingly
pub(crate) fn invert_destination_from_message(metadata: &mut XbiMetadata) {
    let reply_para = metadata.src_para_id;
    let my_para = metadata.dest_para_id;

    // TODO: receiver may want to customise the response metadata at some point
    metadata.dest_para_id = reply_para;
    metadata.src_para_id = my_para;
}

// TODO[style]: migrate to `From` for XbiResult
/// Map a result from an xbi handler to a result
pub(crate) fn handler_to_xbi_result<Emitter: ChannelProgressionEmitter>(
    xbi_id: &Vec<u8>,
    info: &HandlerInfo<frame_support::weights::Weight>,
    msg: &mut XbiFormat,
) -> XbiResult {
    Emitter::emit_instruction_handled(msg, &info.weight);

    msg.metadata.fees.push_aggregate(info.weight.into());

    let status: Status = Status::from(&msg.metadata.fees);

    log::debug!(target: "frame-receiver", "XBI handler status: {:?} for id {:?}", status, xbi_id);

    XbiResult {
        id: xbi_id.encode(),
        status,
        output: info.output.clone(),
        ..Default::default()
    }
}

// TODO[Style]: Move to From implementation
pub(crate) fn instruction_error_to_xbi_result(
    xbi_id: &Vec<u8>,
    err: &sp_runtime::DispatchErrorWithPostInfo<frame_support::weights::PostDispatchInfo>,
) -> XbiResult {
    log::error!(target: "frame-receiver", "Failed to execute instruction: {:?}", err);
    XbiResult {
        id: xbi_id.encode(),
        status: Status::FailedExecution,
        output: err.encode(),
        ..Default::default()
    }
}

// TODO[style]: migrate to `From` for XbiResult
pub(crate) fn handler_to_dispatch_info(
    r: Result<HandlerInfo<frame_support::weights::Weight>, DispatchErrorWithPostInfo>,
) -> DispatchResultWithPostInfo {
    r.map(|info| PostDispatchInfo {
        actual_weight: Some(info.weight),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use crate::frame::instruction_error_to_xbi_result;

    use super::{handler_to_xbi_result, invert_destination_from_message};

    use codec::Encode;
    use frame_support::{dispatch::DispatchErrorWithPostInfo, weights::PostDispatchInfo};
    use xbi_channel_primitives::{traits::HandlerInfo, XbiFormat, XbiMetadata};
    use xbi_format::{Fees, Status};

    #[test]
    fn inverting_destination_works_correctly_when_within_gas() {
        let mut metadata = XbiMetadata {
            src_para_id: 1,
            dest_para_id: 2,
            ..Default::default()
        };
        assert_eq!(metadata.src_para_id, 1);
        assert_eq!(metadata.dest_para_id, 2);
        invert_destination_from_message(&mut metadata);
        assert_eq!(metadata.src_para_id, 2);
        assert_eq!(metadata.dest_para_id, 1);
    }

    #[test]
    fn xbi_handler_maps_to_result_correctly_when_exceeded_gas() {
        let id = b"hello".to_vec();

        let info = HandlerInfo {
            weight: 100,
            output: b"world".to_vec(),
        };

        let mut msg = XbiFormat {
            metadata: XbiMetadata {
                fees: Fees::new(None, Some(1), Some(10_000_000_000)),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = handler_to_xbi_result::<()>(&id, &info, &mut msg);

        assert_eq!(msg.metadata.fees.aggregated_cost, 100);
        assert_eq!(result.id, id.encode());
        assert_eq!(result.status, Status::ExecutionLimitExceeded);
    }

    #[test]
    fn xbi_handler_maps_to_result_correctly() {
        let id = b"hello".to_vec();

        let info = HandlerInfo {
            weight: 100,
            output: b"world".to_vec(),
        };

        let mut msg = XbiFormat {
            metadata: XbiMetadata {
                fees: Fees::new(None, Some(10_000_000_000), Some(10_000_000_000)),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = handler_to_xbi_result::<()>(&id, &info, &mut msg);

        assert_eq!(msg.metadata.fees.aggregated_cost, 100);
        assert_eq!(result.id, id.encode());
        assert_eq!(result.status, Status::Success);
    }

    #[test]
    fn xbi_handler_error_maps_to_result_correctly() {
        let id = b"hello".to_vec();

        let err = DispatchErrorWithPostInfo {
            post_info: PostDispatchInfo {
                actual_weight: Some(1000),
                ..Default::default()
            },
            error: sp_runtime::DispatchError::Other("Fail"),
        };
        let result = instruction_error_to_xbi_result(&id, &err);

        assert_eq!(result.id, id.encode());
        assert_eq!(result.status, Status::FailedExecution);
        assert_eq!(result.output, err.encode());
    }
}
