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

        match &mut msg {
            Message::Request(format) => {
                format.metadata.progress(Submitted(current_block));

                // TODO: charge as reserve because we pay as sovereign
                // TODO: actually reserve fees

                Queue::push((
                    Message::Request(format.to_owned()),
                    QueueSignal::PendingRequest,
                ));
            }
            Message::Response(result, metadata) => {
                // Progress the delivered timestamp
                metadata.progress(Responded(current_block));

                // TODO: charge as reserve because we pay as sovereign
                // TODO: actually reserve fees

                Queue::push((
                    Message::Response(result.to_owned(), metadata.to_owned()),
                    QueueSignal::PendingResponse,
                ));
            }
        }
        Ok(())
    }
}
