use crate::receiver::Receiver as ReceiverExt;
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_system::{ensure_signed, Config};
use sp_runtime::{traits::UniqueSaturatedInto, Either};
use sp_std::marker::PhantomData;
use xp_channel::{
    queue::{QueueSignal, Queueable},
    ChannelProgressionEmitter, Message,
};
use xp_format::Timestamp::*;
use xp_format::{XbiFormat, XbiMetadata, XbiResult};

/// This is an asynchronous queue backed frame receiver, which expects some queue handler to transport the messages back via the transport layer,
/// detaching the message handling part with the transport of the message.
pub struct Receiver<T, Queue, Emitter> {
    phantom: PhantomData<(T, Queue, Emitter)>,
}

impl<T, Queue, Emitter> ReceiverExt for Receiver<T, Queue, Emitter>
where
    T: Config,
    Queue: Queueable<(Message, QueueSignal)>,
    Emitter: ChannelProgressionEmitter,
{
    type Origin = T::Origin;
    type Outcome = DispatchResultWithPostInfo;

    /// Request should always run the instruction, and produce some info containing meters for the execution
    fn handle_request(origin: &Self::Origin, format: &mut XbiFormat) -> DispatchResultWithPostInfo {
        let _who = ensure_signed(origin.clone())?;
        Emitter::emit_received(Either::Left(format));

        let current_block: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();

        // progress to delivered
        format.metadata.progress(Delivered(current_block));

        Queue::push((
            Message::Request(format.to_owned()),
            QueueSignal::PendingExecution,
        ));

        // TODO: cost of queueing message
        Ok(Default::default())
    }

    /// Response should delegate to the queue handler who would know about how to handle the message
    fn handle_response(
        origin: &Self::Origin,
        msg: &XbiResult,
        metadata: &XbiMetadata,
    ) -> DispatchResultWithPostInfo {
        let _who = ensure_signed(origin.clone())?;
        Emitter::emit_received(Either::Right(msg));

        Queue::push((
            Message::Response(msg.clone(), metadata.clone()),
            QueueSignal::PendingResult,
        ));

        // TODO: add the cost of handling this response here
        Ok(Default::default())
    }
}
