use crate::{sp_std::marker::PhantomData, Sender, Vec};
use frame_system::Config;
use sp_runtime::{traits::UniqueSaturatedInto, DispatchError, DispatchResult};
use xbi_channel_primitives::{
    queue::{QueueSignal, Queueable},
    ChannelProgressionEmitter, Message,
};
use xcm::prelude::*;
use xcm_primitives::{MultiLocationBuilder, XcmBuilder};

pub struct FrameSender<T, Emitter, CallProvider, Xcm, Call, Queue, AssetRegistry, AssetId> {
    #[allow(clippy::all)]
    phantom: PhantomData<(
        T,
        Emitter,
        CallProvider,
        Xcm,
        Call,
        Queue,
        AssetRegistry,
        AssetId,
    )>,
}

/// A trait requiring some implementer to provide a way for us to transform some type T
/// into a dispatchable call to receiver
///
/// The implementer should implement this trait since it cannot be known by this crate, e.g:
///
/// ```
/// # use std::collections::VecDeque;
/// # pub fn test<T: frame_system::Config>(format: xbi_channel_primitives::XbiFormat) {
///     //let pallet_index_in_runtime = 200;
///     //let mut xbi_call: VecDeque<u8> =
///     //   crate::pallet::Call::receive::<T> { msg: format.into() }
///     //       .encode()
///     //       .into();
///     //xbi_call.push_front(pallet_index_in_runtime); // Pallet index is not known by the crate for every channel and can be changed anytime
/// # }
/// ```
pub trait ReceiveCallProvider {
    fn provide<T: Into<Message>>(t: T) -> Vec<u8>;
}

impl<T, Emitter, CallProvider, Xcm, Call, Queue, AssetLookup, AssetId> Sender<Message>
    for FrameSender<T, Emitter, CallProvider, Xcm, Call, Queue, AssetLookup, AssetId>
where
    T: Config,
    Emitter: ChannelProgressionEmitter,
    CallProvider: ReceiveCallProvider,
    Xcm: SendXcm,
    Queue: Queueable<(Message, QueueSignal)>,
    AssetLookup: xcm_primitives::frame_traits::AssetLookup<AssetId>,
    AssetId: From<u32> + Clone,
{
    type Outcome = DispatchResult;

    fn send(mut msg: Message) -> Self::Outcome {
        let current_block: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();

        let metadata = msg.get_metadata().clone();

        let dest = MultiLocationBuilder::new_parachain(metadata.dest_para_id)
            .with_parents(1)
            .build();

        match &mut msg {
            Message::Request(format) => {
                // Progress the submitted timestamp
                format.metadata.timesheet.progress(current_block);

                // TODO: we need to do more here, we need the store, and we need to be able to queue/enqueue this, atm it is just 1 for 1

                // TODO: if dest is self, just instant checkout
                // TODO: this needs to determine whether its on the sender (src == dest) -> self::receive
                // If ordered execution locally via XBI : (T::MyParachainId::get(), T::MyParachainId::get())
                // Or if received XBI order of execution from remote Parachain

                // Progress the sent timestamp, in hope of being sent
                format.metadata.timesheet.progress(current_block);

                // TODO: charge as reserve because we pay as sovereign
                // TODO: actually reserve fees
                let payment_asset = match format.metadata.fees.asset {
                    Some(id) => {
                        let id: AssetId = id.into();
                        AssetLookup::reverse_ref(&id).map_err(|_| DispatchError::CannotLookup)?
                    },
                    None => MultiLocationBuilder::new_native().build(),
                };

                let xbi_format_msg = XcmBuilder::<()>::default()
                    .with_withdraw_concrete_asset(
                        payment_asset.clone(),
                        format.metadata.fees.max_notifications_cost,
                    )
                    .with_buy_execution(payment_asset, 1_000_000_000, None) // TODO: same as above
                    .with_transact(
                        Some(OriginKind::SovereignAccount),
                        Some(metadata.fees.max_notifications_cost as u64),
                        CallProvider::provide(format.clone()),
                    )
                    .build();

                // TODO: determine if this needs sudo or we use something like a hook to do all of these for us
                Xcm::send_xcm(dest, xbi_format_msg)
                    .map(|_| {
                        Emitter::emit_sent(msg.clone());
                        // Pending result, queue will persist the message
                        Queue::push((msg.clone(), QueueSignal::PendingResponse));
                    })
                    .map_err(|e| {
                        Queue::push((msg, QueueSignal::XcmSendError));
                        // <XbiCheckOutsQueued<T>>::insert(
                        //     xbi_id.clone(),
                        //     XbiCheckOut::new_ignore_costs::<T>(
                        //         xbi_id.encode(),
                        //         format.notification_delivery_timeout,
                        //         e.encode(),
                        //         XbiCheckOutStatus::ErrorFailedOnXCMDispatch,
                        //     ),
                        // );
                        log::error!(target: "xbi-sender", "Failed to send xcm request: {:?}", e);
                        DispatchError::Other("Failed to send xcm request")
                    })
            },
            Message::Response(result, metadata) => {
                // Progress the delivered timestamp
                metadata.timesheet.progress(current_block);

                // TODO: if sending response to self, then handle immediately
                // TODO: determine costs
                // TODO: apply costs

                // TODO: Set this and get it from config
                let require_weight_at_most = 1_000_000_000;

                // NOTE: do we want to allow the user to control what asset we pay for in response?
                // I think that should be configured by the channel implementation, not the user
                //
                //
                let _payment_asset = match metadata.fees.asset {
                    Some(id) => {
                        let id: AssetId = id.into();
                        AssetLookup::reverse_ref(&id).map_err(|_| DispatchError::CannotLookup)?
                    },
                    None => MultiLocationBuilder::new_native().build(),
                };

                let xbi_format_msg = XcmBuilder::<()>::default()
                    // TODO: reenable based on above conversations
                    // .with_withdraw_concrete_asset(payment_asset.clone(), 1_000_000_000_000) // TODO: take amount from new costs field
                    // .with_buy_execution(payment_asset, 1_000_000_000, None) // TODO: same as above
                    .with_transact(
                        Some(OriginKind::SovereignAccount),
                        Some(require_weight_at_most),
                        CallProvider::provide((result.clone(), metadata.clone())),
                    )
                    .build();

                Xcm::send_xcm(dest, xbi_format_msg)
                    .map(|_| Emitter::emit_sent(msg.clone()))
                    .map_err(|e| {
                        Queue::push((msg, QueueSignal::XcmSendError));
                        log::error!(target: "xbi-sender", "Failed to send xcm request: {:?}", e);
                        DispatchError::Other("Failed to send xcm request")
                    })
            },
        }
    }
}
