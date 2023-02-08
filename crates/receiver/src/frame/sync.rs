use crate::{
    frame::{
        handler_to_dispatch_info, handler_to_xbi_result, instruction_error_to_xbi_result,
        invert_destination_from_message,
    },
    Receiver as ReceiverExt,
};
use codec::Encode;
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_system::{ensure_signed, Config};
use sp_runtime::{traits::UniqueSaturatedInto, Either};
use sp_std::marker::PhantomData;
use xbi_channel_primitives::{
    queue::{QueueSignal, Queueable},
    traits::XbiInstructionHandler,
    ChannelProgressionEmitter, Message,
};
use xbi_format::{XbiFormat, XbiMetadata, XbiResult};

/// This is a synchronous backed Frame receiver
/// It services the `REQ-REP` side of an async channel, that is to say, it receives a message, handles it, then responds with the result
/// all in one step.
pub struct Receiver<T, Sender, Emitter, Queue, InstructionHandler> {
    phantom: PhantomData<(T, Sender, Emitter, Queue, InstructionHandler)>,
}

#[cfg(feature = "frame")]
impl<T, Sender, Emitter, Queue, InstructionHandler> ReceiverExt
    for Receiver<T, Sender, Emitter, Queue, InstructionHandler>
where
    T: Config,
    Sender: xbi_sender::Sender<Message>,
    Emitter: ChannelProgressionEmitter,
    Queue: Queueable<(Message, QueueSignal)>,
    InstructionHandler: XbiInstructionHandler<T::Origin>,
{
    type Origin = T::Origin;
    type Outcome = DispatchResultWithPostInfo;

    /// Request should always run the instruction, and product some dispatch info containing meters for the execution
    fn handle_request(origin: &Self::Origin, msg: &mut XbiFormat) -> DispatchResultWithPostInfo {
        let _who = ensure_signed(origin.clone())?;
        let current_block: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();

        msg.metadata.timesheet.progress(current_block);

        Emitter::emit_received(Either::Left(msg));

        invert_destination_from_message(&mut msg.metadata);

        let xbi_id = msg.metadata.id::<T::Hashing>();

        let instruction_handle = InstructionHandler::handle(origin, msg);

        let xbi_result = match &instruction_handle {
            Ok(info) => handler_to_xbi_result::<Emitter>(&xbi_id.encode(), info, msg),
            Err(e) => instruction_error_to_xbi_result(&xbi_id.encode(), e),
        };

        msg.metadata.timesheet.progress(current_block);

        Emitter::emit_request_handled(
            &xbi_result,
            &msg.metadata,
            &match &instruction_handle {
                Ok(info) => Some(info.weight),
                Err(e) => e.post_info.actual_weight,
            }
            .unwrap_or_default(),
        );

        Sender::send(Message::Response(xbi_result, msg.metadata.clone()));

        handler_to_dispatch_info(instruction_handle)
    }

    // TODO: this should not have a queue anymore, we should provide some storage interface to write the result and add the cost.
    /// Response should update the state of the storage checkout queues and notify the sender of completion
    fn handle_response(
        origin: &Self::Origin,
        msg: &XbiResult,
        _metadata: &XbiMetadata,
    ) -> DispatchResultWithPostInfo {
        let _who = ensure_signed(origin.clone())?;
        Emitter::emit_received(Either::Right(msg));

        // TODO: add the cost of handling this response here
        Ok(Default::default())
    }
}
