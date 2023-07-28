use super::ReceiveCallProvider;
use crate::sender::Sender as SenderExt;
use codec::{Decode, Encode};
use frame_support::traits::{
    fungibles::{Inspect, Mutate},
    Get, ReservableCurrency,
};
use frame_system::Config;
use sp_runtime::{traits::UniqueSaturatedInto, DispatchError, DispatchResult};
use sp_std::marker::PhantomData;
use xp_channel::{ChannelProgressionEmitter, Message, SendXcm};
use xp_format::Timestamp::*;
use xp_xcm::{xcm::prelude::*, MultiLocationBuilder, XcmBuilder};

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

        let metadata = msg.get_metadata().clone();

        let mut dest = MultiLocationBuilder::new_parachain(metadata.dest_para_id)
            .with_parents(1)
            .build();

        match &mut msg {
            Message::Request(format) => {
                // Progress the timestamps in one go
                format.metadata.progress(Submitted(current_block));
                format.metadata.progress(Sent(current_block));

                let o: T::AccountId = crate::xbi_origin(&format.metadata)?;
                ChargeForMessage::charge(&o, &format.metadata.fees)?;

                let payment_asset = match format.metadata.fees.asset {
                    Some(id) => {
                        let id: AssetIdOf<T, Assets> = Decode::decode(&mut &id.encode()[..])
                            .map_err(|_| DispatchError::CannotLookup)?;
                        AssetLookup::reverse_ref(id).map_err(|_| DispatchError::CannotLookup)?
                    },
                    None => MultiLocationBuilder::new_native().build(),
                };

                let xbi_format_msg = XcmBuilder::<()>::default()
                    .with_withdraw_concrete_asset(
                        payment_asset.clone(),
                        format.metadata.fees.get_aggregated_limit(),
                    )
                    .with_buy_execution(
                        payment_asset,
                        format.metadata.fees.notification_cost_limit,
                        None,
                    )
                    .with_transact(
                        Some(OriginKind::SovereignAccount),
                        None,
                        CallProvider::provide(format.clone()),
                    )
                    // TODO deposit whatever is left over in the reserve
                    // .with_deposit_asset(
                    //     MultiLocationBuilder::new_parachain(format.metadata.src_para_id).build(),
                    //     50,
                    // )
                    .build();

                // Xcm::send_xcm(dest, xbi_format_msg).map(|_| {
                //     log::trace!(target: "xs-channel", "Successfully sent xcm message with hash {:?}", xcm_hash);
                //     Emitter::emit_sent(msg.clone());
                // }).map_err(|e| {
                //     // TODO: ensure happens on queue
                //     if let Err(e) = ChargeForMessage::refund(&o, &metadata.fees) {
                //         println!(target: "xs-channel", "Failed to refund fees: {:?}", e);
                //     }
                //
                //     println!(target: "xs-channel", "Failed to send xcm request: {:?}", e);
                //     DispatchError::Other("Failed to send xcm request")
                // })

                Xcm::validate(&mut Some(dest), &mut Some(xbi_format_msg))
                    // TODO: now we know the fees before we send the message, update ChargeForAsset to be XCMv3 Friendly
                    .and_then(|(ticket, fees_for_message)| Xcm::deliver(ticket))
                    .map(|xcm_hash| {
                        log::trace!(target: "xs-channel", "Successfully sent xcm message with hash {:?}", xcm_hash);
                        println!(
                            "DUPA EMIT Successfully sent xcm message with hash {:?}",
                            xcm_hash
                        );
                        Emitter::emit_sent(msg.clone());
                    })
                    .map_err(|e| {
                        // TODO: ensure happens on queue
                        if let Err(e) = ChargeForMessage::refund(&o, &metadata.fees) {
                            println!( "Failed to refund fees: {:?}", e);
                        }

                        println!( "Failed to send xcm request: {:?}", e);
                        DispatchError::Other("Failed to send xcm request")
                    })
            },
            Message::Response(result, metadata) => {
                // Progress the delivered timestamp
                metadata.progress(Responded(current_block));

                // Sovereign will pay for this on behalf of the user, this is tricky to get
                // TODO: Set this and get it from config
                let require_weight_at_most = 1_000_000_000;

                // NOTE: do we want to allow the user to control what asset we pay for in response?
                // I think that should be configured by the channel implementation, not the user
                let _payment_asset = match metadata.fees.asset {
                    Some(id) => {
                        let id: AssetIdOf<T, Assets> = Decode::decode(&mut &id.encode()[..])
                            .map_err(|_| DispatchError::CannotLookup)?;
                        AssetLookup::reverse_ref(id).map_err(|_| DispatchError::CannotLookup)?
                    },
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

                // Xcm::send_xcm(dest, xbi_format_msg).map(|_| {
                //     log::trace!(target: "xs-channel", "Successfully sent xcm message with hash {:?}", xcm_hash);
                //     Emitter::emit_sent(msg.clone());
                // }).map_err(|e| {
                //     // TODO: ensure happens on queue
                //     if let Err(e) = ChargeForMessage::refund(&o, &metadata.fees) {
                //         println!(target: "xs-channel", "Failed to refund fees: {:?}", e);
                //     }
                //
                //     println!(target: "xs-channel", "Failed to send xcm request: {:?}", e);
                //     DispatchError::Other("Failed to send xcm request")
                // })

                Xcm::validate(&mut Some(dest), &mut Some(xbi_format_msg))
                    // TODO: now we know the fees before we send the message, update ChargeForAsset to be XCMv3 Friendly
                    .and_then(|(ticket, fees_for_message)| Xcm::deliver(ticket))
                    .map(|xcm_hash| {
                        println!(
                            "DUPA EMIT Successfully sent xcm message with hash {:?}",
                            xcm_hash
                        );
                        Emitter::emit_sent(msg.clone())
                    })
                    .map_err(|e| {
                        println!("Failed to send xcm request: {:?}", e);
                        DispatchError::Other("Failed to send xcm request")
                    })
            },
        }
    }
}
