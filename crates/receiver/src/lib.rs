use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;
use xbi_format::{XBICheckIn, XBICheckOut, XBICheckOutStatus, XBIFormat, XBIInstr};

#[cfg(feature = "frame")]
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
#[cfg(feature = "frame")]
use frame_system::{ensure_signed, Config};

/// Receivers either handle requests (checkins) or responses(checkouts)
#[derive(Clone, Eq, PartialEq, Encode, Decode, TypeInfo, Debug)]
pub enum Message<BlockNumber> {
    // #[cfg(feature = "requests")]
    CheckIn(XBICheckIn<BlockNumber>),
    // #[cfg(feature = "responses")]
    CheckOut(XBICheckOut),
}

pub trait Receiver<BlockNumber> {
    type Origin;
    type Outcome;

    /// Receive an xbi request
    fn receive(origin: Self::Origin, msg: Message<BlockNumber>) -> Self::Outcome {
        match msg {
            // #[cfg(feature = "requests")]
            Message::CheckIn(msg) => Self::handle_request(origin, msg),
            // #[cfg(feature = "responses")]
            Message::CheckOut(msg) => Self::handle_response(origin, msg),
        }
    }

    /// Request should always run the instruction, and product some dispatch info containing meters for the execution
    fn handle_request(origin: Self::Origin, msg: XBICheckIn<BlockNumber>) -> Self::Outcome;

    /// This enables the receiver to select which instructions it wants to handle
    fn handle_instruction(origin: Self::Origin, xbi: XBIFormat) -> Self::Outcome;

    /// Response should update the state of the storage checkout queues and notify the sender of completion
    fn handle_response(caller: Self::Origin, msg: XBICheckOut) -> Self::Outcome;
}

#[cfg(feature = "frame")]
pub struct FrameReceiver<T, Sender> {
    phantom: PhantomData<(T, Sender)>,
}

#[cfg(feature = "frame")]
pub type ParachainTargetId = u32;

#[cfg(feature = "frame")]
impl<T: Config, Sender: xbi_sender::Sender<(ParachainTargetId, Message<T::BlockNumber>)>>
    Receiver<T::BlockNumber> for FrameReceiver<T, Sender>
{
    type Origin = T::Origin;
    type Outcome = DispatchResultWithPostInfo;

    /// Request should always run the instruction, and product some dispatch info containing meters for the execution
    fn handle_request(
        origin: Self::Origin,
        msg: XBICheckIn<T::BlockNumber>,
    ) -> DispatchResultWithPostInfo {
        let _who = ensure_signed(origin.clone())?;
        let xbi_id = msg.xbi.metadata.id::<T::Hashing>();
        let dest = msg.xbi.metadata.src_para_id;

        match Self::handle_instruction(origin, msg.xbi.clone()) {
            Ok(info) => {
                // todo: source for the cost of XBI Dispatch - execute in credit
                let actual_delivery_cost = 0;
                let execution_cost = info.actual_weight.map(|w| w.into()).unwrap_or(0);
                Sender::send((
                    dest,
                    // TODO: use scabi to convert postdispatch_info
                    Message::CheckOut(XBICheckOut::new::<T>(
                        msg.notification_delivery_timeout,
                        info.encode(),
                        XBICheckOutStatus::SuccessfullyExecuted,
                        execution_cost,
                        actual_delivery_cost,
                        actual_delivery_cost + execution_cost,
                    )),
                ));
                Ok(info)
            }
            Err(e) => {
                let checkout = XBICheckOut::new_ignore_costs::<T>(
                    msg.notification_delivery_timeout,
                    e.encode(),
                    XBICheckOutStatus::ErrorFailedExecution,
                );
                // TODO: another option is to queue these messages with destinations and the queue knows about the sender/receiver
                // <XBICheckOutsQueued<T>>::insert(xbi_id, checkout);
                Sender::send((dest, Message::CheckOut(checkout)));
                Err(e)
            }
        }
    }

    /// This enables the receiver to select which instructions it wants to handle
    fn handle_instruction(origin: Self::Origin, xbi: XBIFormat) -> DispatchResultWithPostInfo {
        let caller = ensure_signed(origin.clone())?;
        // TODO: provide the callbacks here
        match xbi.instr {
            XBIInstr::Unknown { .. } => {}
            XBIInstr::CallNative { .. } => {}
            XBIInstr::CallEvm { .. } => {}
            XBIInstr::CallWasm { .. } => {}
            XBIInstr::CallCustom { .. } => {}
            XBIInstr::Transfer { .. } => {}
            XBIInstr::TransferAssets { .. } => {}
            XBIInstr::Swap { .. } => {}
            XBIInstr::AddLiquidity { .. } => {}
            XBIInstr::RemoveLiquidity { .. } => {}
            XBIInstr::GetPrice { .. } => {}
            XBIInstr::Result { .. } => {}
        }
        Ok(Default::default()) // TODO: no
    }

    /// Response should update the state of the storage checkout queues and notify the sender of completion
    fn handle_response(origin: Self::Origin, msg: XBICheckOut) -> DispatchResultWithPostInfo {
        Ok(Default::default()) //TODO: no
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receiver() {}
}
