use codec::Encode;

use sp_runtime::sp_std::prelude::*;
use xp_channel::{traits::HandlerInfo, ChannelProgressionEmitter};
use xp_format::{Status, XbiFormat, XbiMetadata, XbiResult};

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

pub(crate) fn handle_instruction_result<E: ChannelProgressionEmitter>(
    instruction_handle: &Result<
        HandlerInfo<frame_support::weights::Weight>,
        sp_runtime::DispatchErrorWithPostInfo<frame_support::weights::PostDispatchInfo>,
    >,
    msg: &mut XbiFormat,
) -> XbiResult {
    match instruction_handle {
        Ok(info) => handler_to_xbi_result::<E>(info, msg),
        Err(e) => instruction_error_to_xbi_result(e),
    }
}

/// Map a result from an xbi handler to a result
pub(crate) fn handler_to_xbi_result<Emitter: ChannelProgressionEmitter>(
    info: &HandlerInfo<frame_support::weights::Weight>,
    msg: &mut XbiFormat,
) -> XbiResult {
    Emitter::emit_instruction_handled(msg, &info.weight);

    msg.metadata.fees.push_aggregate(info.weight.into());

    let status: Status = Status::from(&msg.metadata.fees);

    log::debug!(target: "frame-receiver", "XBI handler status: {:?} for id {:?}", status, msg.metadata.get_id());

    XbiResult {
        status,
        output: info.output.clone(),
        ..Default::default()
    }
}

// TODO[Style]: Move to From implementation
pub(crate) fn instruction_error_to_xbi_result(
    err: &sp_runtime::DispatchErrorWithPostInfo<frame_support::weights::PostDispatchInfo>,
) -> XbiResult {
    log::error!(target: "frame-receiver", "Failed to execute instruction: {:?}", err);
    XbiResult {
        status: Status::FailedExecution,
        output: err.encode(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::receiver::frame::instruction_error_to_xbi_result;

    use super::{handler_to_xbi_result, invert_destination_from_message};

    use codec::Encode;
    use frame_support::{dispatch::DispatchErrorWithPostInfo, weights::PostDispatchInfo};
    use xp_channel::{traits::HandlerInfo, XbiFormat, XbiMetadata};
    use xp_format::{Fees, Status};

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

        let result = handler_to_xbi_result::<()>(&info, &mut msg);

        assert_eq!(msg.metadata.fees.aggregated_cost, 100);
        assert_eq!(result.status, Status::ExecutionLimitExceeded);
    }

    #[test]
    fn xbi_handler_maps_to_result_correctly() {
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

        let result = handler_to_xbi_result::<()>(&info, &mut msg);

        assert_eq!(msg.metadata.fees.aggregated_cost, 100);
        assert_eq!(result.status, Status::Success);
    }

    #[test]
    fn xbi_handler_error_maps_to_result_correctly() {
        let err = DispatchErrorWithPostInfo {
            post_info: PostDispatchInfo {
                actual_weight: Some(1000),
                ..Default::default()
            },
            error: sp_runtime::DispatchError::Other("Fail"),
        };
        let result = instruction_error_to_xbi_result(&err);

        assert_eq!(result.status, Status::FailedExecution);
        assert_eq!(result.output, err.encode());
    }
}
