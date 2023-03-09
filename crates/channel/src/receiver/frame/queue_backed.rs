use crate::receiver::Receiver as ReceiverExt;
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_system::{ensure_signed_or_root, Config};
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
        let _who = ensure_signed_or_root(origin.clone())?;
        format.metadata.progress(Delivered(
            <frame_system::Pallet<T>>::block_number().unique_saturated_into(),
        ));
        Emitter::emit_received(Either::Left(format));

        Queue::push((
            Message::Request(format.to_owned()),
            QueueSignal::PendingExecution,
        ));

        // TODO: cost of queueing message, explore this
        Ok(Default::default())
    }

    /// Response should delegate to the queue handler who would know about how to handle the message
    fn handle_response(
        origin: &Self::Origin,
        msg: &XbiResult,
        metadata: &XbiMetadata,
    ) -> DispatchResultWithPostInfo {
        let _who = ensure_signed_or_root(origin.clone())?;
        let mut meta = metadata.clone();

        meta.progress(Received(
            <frame_system::Pallet<T>>::block_number().unique_saturated_into(),
        ));
        Emitter::emit_received(Either::Right(msg));

        Queue::push((
            Message::Response(msg.clone(), meta.clone()),
            QueueSignal::PendingResult,
        ));

        // TODO: add the cost of handling this response here, explore this
        Ok(Default::default())
    }
}
