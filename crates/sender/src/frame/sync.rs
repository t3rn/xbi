use crate::{sp_std::marker::PhantomData, Sender as SenderExt};
use frame_system::Config;
use sp_runtime::{traits::UniqueSaturatedInto, DispatchError, DispatchResult};
use xbi_channel_primitives::{ChannelProgressionEmitter, Message};
use xcm::prelude::*;
use xcm_primitives::{MultiLocationBuilder, XcmBuilder};

use super::ReceiveCallProvider;

// TODO: currently there is a spike to investigate this via dispatch send round-trip
/// A synchronous frame-based channel sender part.
pub struct Sender<T, Emitter, CallProvider, Xcm, Call, AssetRegistry, AssetId> {
    #[allow(clippy::all)]
    phantom: PhantomData<(T, Emitter, CallProvider, Xcm, Call, AssetRegistry, AssetId)>,
}

impl<T, Emitter, CallProvider, Xcm, Call, AssetLookup, AssetId> SenderExt<Message>
    for Sender<T, Emitter, CallProvider, Xcm, Call, AssetLookup, AssetId>
where
    T: Config,
    Emitter: ChannelProgressionEmitter,
    CallProvider: ReceiveCallProvider,
    Xcm: SendXcm,
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

                // Progress the sent timestamp, in hope of being sent
                format.metadata.timesheet.progress(current_block);

                // TODO: charge as reserve because we pay as sovereign
                // TODO: actually reserve fees
                let payment_asset = match format.metadata.fees.asset {
                    Some(id) => {
                        let id: AssetId = id.into();
                        AssetLookup::reverse_ref(&id).map_err(|_| DispatchError::CannotLookup)?
                    }
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

                Xcm::send_xcm(dest, xbi_format_msg)
                    .map(|_| {
                        log::trace!(target: "xbi-sender", "Successfully sent xcm message");
                        Emitter::emit_sent(msg.clone());
                    })
                    .map_err(|e| {
                        log::error!(target: "xbi-sender", "Failed to send xcm request: {:?}", e);
                        DispatchError::Other("Failed to send xcm request")
                    })
            }
            Message::Response(result, metadata) => {
                // Progress the delivered timestamp
                metadata.timesheet.progress(current_block);

                // TODO: Set this and get it from config
                let require_weight_at_most = 1_000_000_000;

                // NOTE: do we want to allow the user to control what asset we pay for in response?
                // I think that should be configured by the channel implementation, not the user
                let _payment_asset = match metadata.fees.asset {
                    Some(id) => {
                        let id: AssetId = id.into();
                        AssetLookup::reverse_ref(&id).map_err(|_| DispatchError::CannotLookup)?
                    }
                    None => MultiLocationBuilder::new_native().build(),
                };

                let xbi_format_msg = XcmBuilder::<()>::default()
                    // TODO: reenable based on above conversations`
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
                        log::error!(target: "xbi-sender", "Failed to send xcm request: {:?}", e);
                        DispatchError::Other("Failed to send xcm request")
                    })
            }
        }
    }
}
