use std::marker::PhantomData;
use xbi_format::{XBICheckIn, XBICheckOut, XBICheckOutStatus, XBIFormat};

// #[cfg(feature = "frame")]
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_system::{ensure_signed, Config};

/// Receivers either handle requests (checkins) or responses(checkouts)
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

    fn is_invoke_self(src: u32, dest: u32) -> bool {
        src == dest
    }
}

// #[cfg(feature = "frame")]
pub struct FrameReceiver<T: Config> {
    phantom: PhantomData<T>,
}

impl<T: Config> Receiver<T::BlockNumber> for FrameReceiver<T> {
    type Origin = T::Origin;
    type Outcome = DispatchResultWithPostInfo;

    /// Request should always run the instruction, and product some dispatch info containing meters for the execution
    fn handle_request(
        origin: Self::Origin,
        msg: XBICheckIn<T::BlockNumber>,
    ) -> DispatchResultWithPostInfo {
        let _who = ensure_signed(origin.clone())?;
        let xbi_id = msg.xbi.metadata.id::<T::Hashing>();

        match Self::handle_instruction(origin, msg.xbi.clone()) {
            Ok(info) => {
                if Self::is_invoke_self(msg.xbi.metadata.src_para_id, msg.xbi.metadata.dest_para_id)
                {
                    let actual_delivery_cost = 0;
                    // let instant_checkout = XbiAbi::<T>::post_dispatch_info_2_xbi_checkout(
                    //     info,
                    //     checkin.notification_delivery_timeout,
                    //     XBICheckOutStatus::SuccessfullyExecuted,
                    //     actual_delivery_cost,
                    // )?;
                    // <XBICheckOutsQueued<T>>::insert(xbi_id, instant_checkout); TODO: update queued
                    Ok(Default::default()) //TODO: no
                } else {
                    // TODO: send
                    // Self::send((Self::target_2_xcm_location(dest)?.into(), checkin.clone()));
                    Ok(Default::default()) //TODO: no
                }
            }
            Err(e) => {
                // let checkout = XBICheckOut::new_ignore_costs::<T>( TODO: T only needs blocknumber
                //     msg.notification_delivery_timeout,
                //     e.encode(),
                //     XBICheckOutStatus::ErrorFailedExecution,
                // );
                // <XBICheckOutsQueued<T>>::insert(xbi_id, checkout);
                // TODO: fix me, both are wrong
                // Self::send((Self::target_2_xcm_location(dest)?.into(), checkin.clone()));
                Ok(Default::default())
            }
        }
    }

    /// This enables the receiver to select which instructions it wants to handle
    fn handle_instruction(origin: Self::Origin, xbi: XBIFormat) -> DispatchResultWithPostInfo {
        let caller = ensure_signed(origin.clone())?;
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
