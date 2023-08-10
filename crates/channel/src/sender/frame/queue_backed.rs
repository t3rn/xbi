use crate::sender::Sender as SenderExt;
use frame_support::traits::{fungibles::Mutate, Get, ReservableCurrency};
use frame_system::Config;
use sp_runtime::{traits::UniqueSaturatedInto, DispatchResult};
use sp_std::{borrow::ToOwned, marker::PhantomData};
use xp_channel::{
    queue::{QueueSignal, Queueable},
    Message,
};
use xp_format::Timestamp::*;

/// An asynchronous frame-based channel sender part. The resolving messages are handled by some Queue implementation.
pub struct Sender<T, Queue, Currency, Assets, ChargeForMessage, AssetReserveCustodian> {
    #[allow(clippy::all)]
    phantom: PhantomData<(
        T,
        Queue,
        Currency,
        Assets,
        ChargeForMessage,
        AssetReserveCustodian,
    )>,
}

impl<T, Queue, Currency, Assets, ChargeForMessage, AssetReserveCustodian> SenderExt<Message>
    for Sender<T, Queue, Currency, Assets, ChargeForMessage, AssetReserveCustodian>
where
    T: Config,
    Queue: Queueable<(Message, QueueSignal)>,
    Currency: ReservableCurrency<T::AccountId>,
    Assets: Mutate<T::AccountId>,
    ChargeForMessage: xp_channel::traits::MonetaryForMessage<
        T::AccountId,
        Currency,
        Assets,
        AssetReserveCustodian,
    >,
    AssetReserveCustodian: Get<T::AccountId>,
{
    type Outcome = DispatchResult;

    fn send(mut msg: Message) -> Self::Outcome {
        let current_block: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();

        log::debug!(target: "xs-channel", "Sending message: {:?} on block {}", msg, current_block);

        match &mut msg {
            Message::Request(format) => {
                let o: T::AccountId = crate::xbi_origin(&format.metadata)?;

                format.metadata.progress(Submitted(current_block));

                ChargeForMessage::charge(&o, &format.metadata.fees)?;

                log::debug!(target: "xs-channel", "Pushing message: {:?} on block {} to queue", format, current_block);

                Queue::push((
                    Message::Request(format.to_owned()),
                    QueueSignal::PendingRequest,
                ));
            },
            Message::Response(result, metadata) => {
                let o: T::AccountId = crate::xbi_origin(metadata)?;

                metadata.progress(Responded(current_block));

                ChargeForMessage::refund(&o, &metadata.fees)?;

                log::debug!(target: "xs-channel", "Pushing message: {:?} {:?} on block {} to queue", result, metadata, current_block);

                Queue::push((
                    Message::Response(result.to_owned(), metadata.to_owned()),
                    QueueSignal::PendingResponse,
                ));
            },
        }
        Ok(())
    }
}
