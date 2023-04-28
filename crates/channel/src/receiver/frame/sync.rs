use crate::{
    receiver::frame::invert_destination_from_message, Receiver as ReceiverExt, Sender as SenderExt,
};
use frame_support::{
    pallet_prelude::DispatchResultWithPostInfo,
    traits::{fungibles::Mutate, Get, ReservableCurrency},
};
use frame_system::{ensure_signed, Config};
use sp_runtime::{traits::UniqueSaturatedInto, Either};
use sp_std::marker::PhantomData;
use xp_channel::{
    queue::{QueueSignal, Queueable},
    traits::{HandlerInfo, Writable, XbiInstructionHandler},
    ChannelProgressionEmitter, Message,
};
use xp_format::Timestamp::*;
use xp_format::{XbiFormat, XbiMetadata, XbiResult};

use super::handle_instruction_result;

/// This is a synchronous backed Frame receiver
/// It services the `REQ-REP` side of an async channel, that is to say, it receives a message, handles it, then responds with the result
/// all in one step.
pub struct Receiver<
    T,
    Sender,
    Emitter,
    Queue,
    InstructionHandler,
    ResultStore,
    Currency,
    Assets,
    ChargeForMessage,
    AssetReserveCustodian,
> {
    #[allow(clippy::type_complexity)]
    phantom: PhantomData<(
        T,
        Sender,
        Emitter,
        Queue,
        InstructionHandler,
        ResultStore,
        Currency,
        Assets,
        ChargeForMessage,
        AssetReserveCustodian,
    )>,
}

#[cfg(feature = "frame")]
impl<
        T,
        Sender,
        Emitter,
        Queue,
        InstructionHandler,
        ResultStore,
        Currency,
        Assets,
        ChargeForMessage,
        AssetReserveCustodian,
    > ReceiverExt
    for Receiver<
        T,
        Sender,
        Emitter,
        Queue,
        InstructionHandler,
        ResultStore,
        Currency,
        Assets,
        ChargeForMessage,
        AssetReserveCustodian,
    >
where
    T: Config,
    Sender: SenderExt<Message>,
    Emitter: ChannelProgressionEmitter,
    Queue: Queueable<(Message, QueueSignal)>,
    InstructionHandler: XbiInstructionHandler<T::RuntimeOrigin>,
    ResultStore: Writable<(sp_core::H256, XbiResult)>,
    Currency: ReservableCurrency<T::AccountId>,
    Assets: Mutate<T::AccountId>,
    ChargeForMessage:
        xp_channel::traits::RefundForMessage<T::AccountId, Currency, Assets, AssetReserveCustodian>,
    AssetReserveCustodian: Get<T::AccountId>,
{
    type Origin = T::RuntimeOrigin;
    type Outcome = DispatchResultWithPostInfo;

    /// Request should always run the instruction, and product some dispatch info containing meters for the execution
    fn handle_request(origin: &Self::Origin, msg: &mut XbiFormat) -> DispatchResultWithPostInfo {
        let _who = ensure_signed(origin.clone())?;
        msg.metadata.progress(Delivered(
            <frame_system::Pallet<T>>::block_number().unique_saturated_into(),
        ));
        Emitter::emit_received(Either::Left(msg));

        invert_destination_from_message(&mut msg.metadata);

        let instruction_result = InstructionHandler::handle(origin, msg);
        log::debug!(target: "xbi", "Instruction result: {:?}", instruction_result);

        let xbi_result = handle_instruction_result::<Emitter>(&instruction_result, msg);

        log::debug!(target: "xs-channel", "Instruction handled: {:?}", xbi_result);
        msg.metadata.progress(Executed(
            <frame_system::Pallet<T>>::block_number().unique_saturated_into(),
        ));

        Emitter::emit_request_handled(
            &xbi_result,
            &msg.metadata,
            &match &instruction_result {
                Ok(info) => Some(info.weight),
                Err(e) => e.post_info.actual_weight,
            }
            .unwrap_or_default().ref_time(),
        );

        Sender::send(Message::Response(xbi_result, msg.metadata.clone()));

        Ok(instruction_result
            .inspect(|info| log::debug!(target: "xs-channel", "Instruction handled: {:?}", info))
            .map_or_else(|e| e.post_info, HandlerInfo::into))
    }

    fn handle_response(
        origin: &Self::Origin,
        res: &XbiResult,
        metadata: &XbiMetadata,
    ) -> DispatchResultWithPostInfo {
        let _who = ensure_signed(origin.clone())?;
        let mut meta = metadata.clone();

        meta.progress(Received(
            <frame_system::Pallet<T>>::block_number().unique_saturated_into(),
        ));
        Emitter::emit_received(Either::Right(res));

        let o: T::AccountId = crate::xbi_origin(&meta)?;
        ChargeForMessage::refund(&o, &meta.fees)?;

        ResultStore::write((meta.get_id(), res.clone()))
            .map(|_| Default::default())
            .map_err(|e| e.into())
    }
}
