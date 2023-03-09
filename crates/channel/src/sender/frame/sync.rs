use super::ReceiveCallProvider;
use crate::sender::{xbi_origin, Sender as SenderExt};
use codec::{Decode, Encode};
use frame_support::traits::{
    fungibles::{BalancedHold, Inspect, Mutate, MutateHold},
    Get, ReservableCurrency,
};
use frame_system::Config;
use sp_runtime::{traits::UniqueSaturatedInto, DispatchError, DispatchResult};
use sp_std::marker::PhantomData;
use xp_channel::SendXcm;
use xp_channel::{ChannelProgressionEmitter, Message};
use xp_format::Timestamp::*;
use xp_xcm::xcm::prelude::*;
use xp_xcm::{MultiLocationBuilder, XcmBuilder};

// TODO: currently there is a spike to investigate this via dispatch send round-trip
/// A synchronous frame-based channel sender part.
pub struct Sender<
    T,
    Emitter,
    CallProvider,
    Xcm,
    Currency,
    Assets,
    AssetRegistry,
    ChargeForMessage,
    AssetReserveCustodian,
> {
    #[allow(clippy::all)]
    phantom: PhantomData<(
        T,
        Emitter,
        CallProvider,
        Xcm,
        Currency,
        Assets,
        AssetRegistry,
        ChargeForMessage,
        AssetReserveCustodian,
    )>,
}

type AssetIdOf<T, Assets> = <Assets as Inspect<<T as Config>::AccountId>>::AssetId;

impl<
        T,
        Emitter,
        CallProvider,
        Xcm,
        Currency,
        Assets,
        AssetLookup,
        ChargeForMessage,
        AssetReserveCustodian,
    > SenderExt<Message>
    for Sender<
        T,
        Emitter,
        CallProvider,
        Xcm,
        Currency,
        Assets,
        AssetLookup,
        ChargeForMessage,
        AssetReserveCustodian,
    >
where
    T: Config,
    Emitter: ChannelProgressionEmitter,
    CallProvider: ReceiveCallProvider,
    Xcm: SendXcm,
    Currency: ReservableCurrency<T::AccountId>,
    Assets: Mutate<T::AccountId>,
    AssetLookup: xp_xcm::frame_traits::AssetLookup<<Assets as Inspect<T::AccountId>>::AssetId>,
    ChargeForMessage:
        xp_channel::traits::ChargeForMessage<T::AccountId, Currency, Assets, AssetReserveCustodian>,
    AssetReserveCustodian: Get<T::AccountId>,
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
                let o: T::AccountId = xbi_origin(&format.metadata)?;

                // Progress the timestamps in one go
                format.metadata.progress(Submitted(current_block));
                format.metadata.progress(Sent(current_block));

                let o: T::AccountId = crate::xbi_origin(&format.metadata)?;
                ChargeForMessage::charge(&o, &format.metadata.fees)?;

                // TODO: charge as reserve because we pay as sovereign
                // TODO: actually reserve fees
                let payment_asset = match format.metadata.fees.asset {
                    Some(id) => {
                        let id: AssetIdOf<T, Assets> = Decode::decode(&mut &id.encode()[..])
                            .map_err(|_| DispatchError::CannotLookup)?;
                        AssetLookup::reverse_ref(&id).map_err(|_| DispatchError::CannotLookup)?
                    }
                    None => MultiLocationBuilder::new_native().build(),
                };

                let xbi_format_msg = XcmBuilder::<()>::default()
                    .with_withdraw_concrete_asset(
                        payment_asset.clone(),
                        format.metadata.fees.notification_cost_limit,
                    )
                    .with_buy_execution(payment_asset, 1_000_000_000, None) // TODO: same as above
                    .with_transact(
                        Some(OriginKind::SovereignAccount),
                        Some(metadata.fees.execution_cost_limit as u64),
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
                let o: T::AccountId = xbi_origin(&metadata)?;

                // Progress the delivered timestamp
                metadata.progress(Responded(current_block));

                ChargeForMessage::refund(&o, &metadata.fees)?;

                // TODO: Set this and get it from config
                let require_weight_at_most = 1_000_000_000;

                // NOTE: do we want to allow the user to control what asset we pay for in response?
                // I think that should be configured by the channel implementation, not the user
                let _payment_asset = match metadata.fees.asset {
                    Some(id) => {
                        let id: AssetIdOf<T, Assets> = Decode::decode(&mut &id.encode()[..])
                            .map_err(|_| DispatchError::CannotLookup)?;
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
