use crate::sender::Sender as SenderExt;
use frame_system::Config;
use sp_runtime::{traits::UniqueSaturatedInto, DispatchResult};
use sp_std::marker::PhantomData;
use xp_channel::{
    queue::{QueueSignal, Queueable},
    Message,
};
use xp_format::Timestamp::*;

/// An asynchronous frame-based channel sender part. The resolving messages are handled by some Queue implementation.
pub struct Sender<T, Queue> {
    #[allow(clippy::all)]
    phantom: PhantomData<(T, Queue)>,
}

impl<T, Queue> SenderExt<Message> for Sender<T, Queue>
where
    T: Config,
    Queue: Queueable<(Message, QueueSignal)>,
{
    type Outcome = DispatchResult;

    fn send(mut msg: Message) -> Self::Outcome {
        let current_block: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();

        log::debug!(target: "xs-channel", "Sending message: {:?} on block {}", msg, current_block);

        match &mut msg {
            Message::Request(format) => {
                format.metadata.progress(Submitted(current_block));

                assert!(format.metadata.get_timesheet().submitted.is_some());
                // TODO: charge as reserve because we pay as sovereign
                // TODO: actually reserve fees

                log::debug!(target: "xs-channel", "Pushing message: {:?} on block {} to queue", format, current_block);
                println!(
                    "Pushing message: {:?} on block {} to queue",
                    format, current_block
                );
                Queue::push((
                    Message::Request(format.to_owned()),
                    QueueSignal::PendingRequest,
                ));
            }
            Message::Response(result, metadata) => {
                metadata.progress(Responded(current_block));

                // TODO: charge as reserve because we pay as sovereign
                // TODO: actually reserve fees

                log::debug!(target: "xs-channel", "Pushing message: {:?} {:?} on block {} to queue", result, metadata, current_block);

                Queue::push((
                    Message::Response(result.to_owned(), metadata.to_owned()),
                    QueueSignal::PendingResponse,
                ));
            }
        }
        Ok(())
    }
}
