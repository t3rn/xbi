use codec::Encode;
use frame_support::{
    dispatch::DispatchErrorWithPostInfo, pallet_prelude::DispatchResultWithPostInfo,
    weights::PostDispatchInfo,
};
use xbi_channel_primitives::{traits::HandlerInfo, ChannelProgressionEmitter};
use xbi_format::{XbiCheckOutStatus, XbiFormat, XbiMetadata, XbiResult};

pub mod queue_backed;
pub mod sync;

pub(crate) fn invert_destination_from_message(metadata: &mut XbiMetadata) {
    let reply_para = metadata.src_para_id;
    let my_para = metadata.dest_para_id;

    // TODO: receiver may want to customise the response metadata at some point
    metadata.dest_para_id = reply_para;
    metadata.src_para_id = my_para;
}

// TODO[style]: migrate to `From` for XbiResult
pub(crate) fn handler_to_xbi_result<Emitter: ChannelProgressionEmitter>(
    xbi_id: &Vec<u8>,
    info: &HandlerInfo,
    msg: &mut XbiFormat,
) -> XbiResult {
    Emitter::emit_instruction_handled(msg, &info.weight);

    let execution_cost = info.weight.into();
    msg.metadata.fees.actual_aggregated_cost = Some(
        msg.metadata
            .fees
            .actual_aggregated_cost
            .map(|c| c.checked_add(execution_cost).unwrap_or(c))
            .unwrap_or(execution_cost),
    );

    if execution_cost > msg.metadata.fees.max_exec_cost {
        XbiResult {
            id: xbi_id.encode(),
            status: XbiCheckOutStatus::ErrorExecutionCostsExceededAllowedMax,
            output: info.output.clone(),
            ..Default::default()
        }
    } else {
        XbiResult {
            id: xbi_id.encode(),
            status: XbiCheckOutStatus::SuccessfullyExecuted,
            output: info.output.clone(),
            ..Default::default()
        }
    }
}

// TODO[style]: migrate to `From` for XbiResult
pub(crate) fn instruction_error_to_xbi_result(
    xbi_id: &Vec<u8>,
    err: &sp_runtime::DispatchErrorWithPostInfo<frame_support::weights::PostDispatchInfo>,
) -> XbiResult {
    log::error!(target: "frame-receiver", "Failed to execute instruction: {:?}", err);
    XbiResult {
        id: xbi_id.encode(),
        status: XbiCheckOutStatus::ErrorFailedExecution,
        output: err.encode(),
        ..Default::default()
    }
}

pub(crate) fn handler_to_dispatch_info(
    r: Result<HandlerInfo, DispatchErrorWithPostInfo>,
) -> DispatchResultWithPostInfo {
    r.map(|info| PostDispatchInfo {
        actual_weight: Some(info.weight),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xbi_handler_maps_to_result_correctly() {}
}
