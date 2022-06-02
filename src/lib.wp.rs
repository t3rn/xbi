#![cfg_attr(not(feature = "std"), no_std)]

use xcm::latest::Xcm;

pub mod xbi_format;

pub mod primitives;

pub use pallet::*;

pub use xcm::latest;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

#[frame_support::pallet]
pub mod pallet {
    use crate::{
        primitives::{assets::Assets, evm::Evm, orml::ORML, wasm::WASM},
        xbi_format::*,
        *,
    };
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_core::Hasher;
    use sp_std::{default::Default, prelude::*};
    use xcm::latest::{prelude::*, MultiLocation, OriginKind};


    /// Queue XBI for batch execution
    #[pallet::storage]
    pub type XBICheckInsQueued<T> = StorageMap<
        _,
        Identity,
        <T as frame_system::Config>::Hash,
        XBICheckIn<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;

    /// Processed XBI queue pending for execution
    #[pallet::storage]
    pub type XBICheckInsPending<T> = StorageMap<
        _,
        Identity,
        <T as frame_system::Config>::Hash,
        XBICheckIn<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// XBI called for execution
    pub type XBICheckIns<T> = StorageMap<
        _,
        Identity,
        <T as frame_system::Config>::Hash,
        XBICheckIn<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Lifecycle: If executed: XBICheckInsPending -> XBICheckIns -> XBICheckOutsQueued
    /// Lifecycle: If not executed: XBICheckInsPending -> XBICheckOutsQueued
    pub type XBICheckOutsQueued<T> =
        StorageMap<_, Identity, <T as frame_system::Config>::Hash, XBICheckOut, OptionQuery>;

    #[pallet::storage]
    /// XBI Results of execution on local (here) Parachain
    pub type XBICheckOuts<T> =
        StorageMap<_, Identity, <T as frame_system::Config>::Hash, XBICheckOut, OptionQuery>;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_xcm::Config + pallet_balances::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Call: From<Call<Self>>;

        type Evm: Evm<Self>;

        type ORML: ORML<Self>;

        type Assets: Assets<Self>;

        type WASM: WASM<Self>;

        #[pallet::constant]
        type ExpectedBlockTimeMs: Get<u32>;

        #[pallet::constant]
        type CheckInterval: Get<u32>;

        #[pallet::constant]
        type TimeoutChecksLimit: Get<u32>;

        #[pallet::constant]
        type CheckInLimit: Get<u32>;

        #[pallet::constant]
        type CheckOutLimit: Get<u32>;

        #[pallet::constant]
        type MyParachainId: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // `on_initialize` is executed at the beginning of the block before any extrinsic are
        // dispatched.
        //
        // This function must return the weight consumed by `on_initialize` and `on_finalize`.
        fn on_initialize(n: T::BlockNumber) -> Weight {
            // Start with discovering timeout check ins
            if n % T::CheckInterval::get() == T::BlockNumber::from(0u8) {
                // Process checkins
                let mut timeout_counter: u32 = 0;
                // Go over all unfinished Pending and Sent XBI Orders to find those that timed out
                for (xbi_id, xbi_checkin) in <XBICheckInsQueued<T>>::iter() {
                    if T::BlockNumber::from(xbi_checkin.notification_delivery_timeout.clone())
                        > frame_system::Pallet::<T>::block_number()
                    {
                        if timeout_counter > T::TimeoutChecksLimit::get() {
                            break
                        }
                        // XBI Result didn't arrive in expected time.
                        <XBICheckInsQueued<T>>::remove(xbi_id.clone());
                        <XBICheckIns<T>>::insert(xbi_id.clone(), xbi_checkin.clone());
                        <XBICheckOutsQueued<T>>::insert(xbi_id, XBICheckOut::new(
                            xbi_checkin.notification_delivery_timeout,
                            vec![],
                            XBICheckOutStatus::ErrorDeliveryTimeout
                        ));
                        timeout_counter += 1;
                    }
                }
                for (xbi_id, xbi_checkin) in <XBICheckInsPending<T>>::iter() {
                    if T::BlockNumber::from(xbi_checkin.notification_execution_timeout.clone())
                        > frame_system::Pallet::<T>::block_number()
                    {
                        if timeout_counter > T::TimeoutChecksLimit::get() {
                            break
                        }
                        // XBI Result didn't arrive in expected time.
                        <XBICheckInsPending<T>>::remove(xbi_id.clone());
                        <XBICheckIns<T>>::insert(xbi_id.clone(), xbi_checkin.clone());
                        <XBICheckOutsQueued<T>>::insert(xbi_id, XBICheckOut::new(
                            xbi_checkin.notification_delivery_timeout,
                            vec![],
                            XBICheckOutStatus::ErrorExecutionTimeout
                        ));
                        timeout_counter += 1;
                    }
                }

                // Process CheckIn Queue
                let mut checkin_counter: u32 = 0;

                for (xbi_id, xbi_checkout) in <XBICheckInsQueued<T>>::iter() {
                    if checkin_counter > T::CheckInLimit::get() {
                        break
                    }

                    match pallet::Pallet::<T>::enter(xbi_checkin) {
                        // Pending remote XBI execution
                        Ok(None) => {
                            <XBICheckInsPending<T>>::insert(xbi_id, checkout);
                        }
                        // Instant check out - no pending remote XBI execution
                        Ok(Some(checkout)) => {
                            <XBICheckOutsQueued<T>>::insert(xbi_id, checkout);
                        }
                        Err(_err) => {
                            log::info!("Can't enter execution with current XBI - continue and must be handled better");
                        }
                    }
                    <XBICheckInsQueued<T>>::remove(xbi_id.clone());

                    checkout_counter += 1;
                }


                // Process Check Out Queue
                // All XBIs ready to check out (notification, results)
                for (xbi_id, xbi_checkout) in <XBICheckOutsQueued<T>>::iter() {
                    if checkin_counter > T::CheckOutLimit::get() {
                        break
                    }

                    match pallet::Pallet::<T>::exit(xbi_checkout.clone()) {
                        // Pending remote XBI execution
                        Err(_err) => {
                            log::info!("Can't exit execution with current XBI - continue and must be handled better");
                        }
                        _ => { }
                    }
                    <XBICheckOutsQueued<T>>::remove(xbi_id.clone());
                    <XBICheckOuts<T>>::insert(xbi_id.clone(), xbi_checkout);

                    checkout_counter += 1;
                }

                timed_out_checkins
            }

            // Anything that needs to be done at the start of the block.
            // We don't do anything here.
            // ToDo: Do active xtx signals overview and Cancel if time elapsed
            0
        }

        fn on_finalize(_n: T::BlockNumber) {
            // We don't do anything here.

            // if module block number
            // x-t3rn#4: Go over open Xtx and cancel if necessary
        }

        // A runtime code run after every block and have access to extended set of APIs.
        //
        // For instance you can generate extrinsics for the upcoming produced block.
        fn offchain_worker(_n: T::BlockNumber) {
            // We don't do anything here.
            // but we could dispatch extrinsic (transaction/unsigned/inherent) using
            // sp_io::submit_extrinsic

            // // Check every XtxTimeoutCheckInterval blocks
            // if n % T::CheckInInterval::get() == T::BlockNumber::from(0u8) {
            //     let mut checkin_counter: u32 = 0;
            //     // Go over all local XBI orders to dispatch until the allowed limit is exhausted.
            //
            //     let mut xbi_batch: Vec<XBIFormat> = vec![];
            //
            //     for (xbi_ci) in <XBICheckIns<T>>::iter() {
            //         if checkin_counter > T::XBIDispatchLimit::get() {
            //             break
            //         }
            //         checkin_counter += 1;
            //         xbi_batch.push(xbi);
            //     }
            //
            //     let wrapper_batch_enter_xbi = pallet::Call::wrapper_batch_enter_xbi::<T> {
            //         xbi_batch,
            //         dest_para_id: cumulus_primitives_core::ParaId::new(3334),
            //     };
            //
            //     frame_system::offchain::SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
            //         wrapper_batch_enter_xbi.into(),
            //     )
            //         .map_err(|_| {
            //             log::info!("Error::<T>::SubmitTransaction");
            //         });
            //
            // }
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AbiInstructionExecuted,
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        EnterFailedOnXcmSend,
        EnterFailedOnMultiLocationTransform,
        ExitUnhandled,
        XBIInstructionNotAllowedHere,
        XBIAlreadyCheckedIn,
        XBINotificationTimeOutDelivery,
        XBINotificationTimeOutExecution,
        NoXBICallbackSupported,
        NoEVMSupportedAtDest,
        NoWASMSupportedAtDest,
        No3VMSupportedAtDest,
        NoTransferSupportedAtDest,
        NoTransferAssetsSupportedAtDest,
        NoTransferORMLSupportedAtDest,
        NoTransferEscrowSupportedAtDest,
        NoTransferMultiEscrowSupportedAtDest,
        NoSwapSupportedAtDest,
        NoAddLiquiditySupportedAtDest,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn execute_xcm(origin: OriginFor<T>, _xcm: Xcm<Call<T>>) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn execute_xbi(origin: OriginFor<T>, xbi: XBIFormat) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            // ToDo: XBI::Step::1 Auth for XBI origin check
            match xbi.instr {
                XBIInstr::Notification {
                    kind: _,
                    instruction_id: _,
                    extra: _,
                } => {
                    // Self::check_in_instruction(who, xbi)?;
                },
                XBIInstr::CallNative { ref payload } => {
                    // XBI::Step::2 Is the XBI Instruction Allowed on this Parachain
                    Self::check_xbi_instr_allowed_here(XBIInstr::CallNative {
                        payload: payload.to_vec(),
                    })?;
                    // XBI::Step::3 Check in XBI Instruction entry time
                    // Self::check_in_instruction(who, xbi)?;
                    // ToDo: XBI::Step::4 Execute!
                    // let message_call = payload.take_decoded().map_err(|_| Error::FailedToDecode)?;
                    // let actual_weight = match message_call.dispatch(dispatch_origin) {
                    // 	Ok(post_info) => post_info.actual_weight,
                    // 	Err(error_and_info) => {
                    // 		// Not much to do with the result as it is. It's up to the parachain to ensure that the
                    // 		// message makes sense.
                    // 		error_and_info.post_info.actual_weight
                    // 	},
                    // }
                },
                XBIInstr::CallEvm {
                    ref caller,
                    ref dest,
                    ref value,
                    ref input,
                    ref gas_limit,
                    max_fee_per_gas: _,
                    max_priority_fee_per_gas: _,
                    nonce: _,
                    access_list: _,
                } => {
                    // XBI::Step::2 Is the XBI Instruction Allowed on this Parachain
                    Self::check_xbi_instr_allowed_here(XBIInstr::CallEvm {
                        caller: caller.clone(),
                        dest: dest.clone(),
                        value: value.clone(),
                        input: input.clone(),
                        gas_limit: gas_limit.clone(),
                        max_fee_per_gas: None,
                        max_priority_fee_per_gas: None,
                        nonce: None,
                        access_list: None,
                    })?;
                    // XBI::Step::3 Check in XBI Instruction entry time
                    // Self::check_in_instruction(who, xbi)?;
                    // ToDo: XBI::Step::4 Execute!
                    // pallet_evm::Pallet::<T>::call(
                    // 	caller,
                    // 	dest,
                    // 	value,
                    // 	input,
                    // 	gas_limit,
                    // 	max_fee_per_gas,
                    // 	max_priority_fee_per_gas,
                    // 	nonce,
                    // 	access_list,
                    // )
                },
                XBIInstr::CallWasm {
                    ref caller,
                    ref dest,
                    ref value,
                    ref input,
                } => {
                    // XBI::Step::2 Is the XBI Instruction Allowed on this Parachain
                    Self::check_xbi_instr_allowed_here(XBIInstr::CallWasm {
                        caller: caller.clone(),
                        dest: dest.clone(),
                        value: value.clone(),
                        input: input.clone(),
                    })?;
                    // XBI::Step::3 Check in XBI Instruction entry time
                    // Self::check_in_instruction(who, xbi)?;
                    // ToDo: XBI::Step::4 Execute!
                    // pallet_contracts::Pallet::<T>::call(
                    // 	caller,
                    // 	dest,
                    // 	value,
                    // 	input,
                    // )
                },
                XBIInstr::CallCustom { .. } => {},
                XBIInstr::Transfer {
                    ref dest,
                    ref value,
                } => {
                    // XBI::Step::2 Is the XBI Instruction Allowed on this Parachain
                    Self::check_xbi_instr_allowed_here(XBIInstr::Transfer {
                        dest: dest.clone(),
                        value: value.clone(),
                    })?;
                    // XBI::Step::3 Check in XBI Instruction entry time
                    // Self::check_in_instruction(who, xbi)?;
                    // ToDo: XBI::Step::4 Execute!

                    // pallet_balances::Pallet::<T>::transfer(
                    // 	who,
                    // 	dest,
                    // 	value,
                    // )
                },
                XBIInstr::TransferMulti {
                    currency_id: _,
                    ref dest,
                    ref value,
                } => {
                    // XBI::Step::2 Is the XBI Instruction Allowed on this Parachain
                    Self::check_xbi_instr_allowed_here(XBIInstr::TransferMulti {
                        currency_id: Default::default(),
                        dest: dest.clone(),
                        value: value.clone(),
                    })?;
                    // XBI::Step::3 Check in XBI Instruction entry time
                    // Self::check_in_instruction(who, xbi)?;
                    // ToDo: XBI::Step::4 Execute!
                    // pallet_orml_tokens::Pallet::<T>::transfer(
                    // 	currency_id,
                    // 	who,
                    // 	dest,
                    // 	value,
                    // )
                },
                XBIInstr::Result { .. } => {
                    // ToDo! Check out the XBI Instruction and send back the results
                },
            }

            Self::deposit_event(Event::<T>::AbiInstructionExecuted);

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn target_2_xcm_location(target_id: u32) -> Result<xcm::latest::MultiLocation, Error<T>> {
            // Parachain(ParachainInfo::parachain_id().into()).into()
            MultiLocation::try_from(Parachain(target_id.into()).into())
                .map_err(|_| Error::<T>::EnterFailedOnMultiLocationTransform)
        }
            /// Enter might be weight heavy - calls for execution into EVMs and if necessary sends the response
        /// If returns XBICheckOut means that executed instantly and the XBI order can be removed from pending checkouts
        pub fn enter(checkin: XBICheckIn<T::BlockNumber>) -> Result<Option<XBICheckOut>, Error<T>> {
            match (checkin.xbi.metadata.src_para_id, xbi.metadata.dest_para_id) {
                // If ordered execution locally via XBI : (T::MyParachainId::get(), T::MyParachainId::get())
                // Or if received XBI order of execution from remote Parachain
                (_, T::MyParachainId::get()) => {
                    let res = Self::enter_here(checkin.xbi).map_err(|e| {
                        Ok(Some(XBICheckOut::new(
                            checkin.notification_delivery_timeout,
                            e.encode(),
                            XBICheckOutStatus::ErrorFailedExecution,
                        )))
                    })?;
                    // Instant checkout
                    Ok(Some(XBICheckOut::new(
                        checkin.notification_delivery_timeout,
                        res.encode(),
                        XBICheckOutStatus::SuccessfullyExecuted,
                    )))
                },
                // If addressing XBI result back to source Parachain that ordered XBI
                // Or if serving as XBI Router, pass XBI message along
                (_, dest) => {
                    Self::enter_remote(
                        checkin.xbi,
                        Box::new(Self::target_2_xcm_location(dest)?.into()),
                    ).map_err(|e| {
                        Ok(Some(XBICheckOut::new(
                            checkin.notification_delivery_timeout,
                            e.encode(),
                            XBICheckOutStatus::ErrorFailedXCMDispatch,
                        )))
                    })?;
                    // XBI order sent via XCM. Await for XBI notifications.
                    Ok(None)
                },
            }

            // if xbi.metadata.dest_para_id == T::MyParachainId::get() {
            //     Self::enter_here(xbi)
            //         .map_err(|_e| Error::<T>::EnterFailedOnMultiLocationTransform.into())?;
            //     // ToDo: Handle _res: PostDispatchInfo to get actual fees
            //     return Ok(())
            // } else {
            //     let dest = Self::target_2_xcm_location(xbi.metadata.dest_para_id)?;
            //     Self::enter_remote(xbi, Box::new(dest.into()))
            // }
        }

        fn exit_remote(
            xbi: XBIFormat,
            dest: Box<xcm::VersionedMultiLocation>,
        ) -> Result<(), Error<T>> {
            Ok(())
        }

        fn enter_remote(
            xbi: XBIFormat,
            dest: Box<xcm::VersionedMultiLocation>,
        ) -> Result<(), Error<T>> {
            let dest = MultiLocation::try_from(*dest)
                .map_err(|()| Error::<T>::EnterFailedOnMultiLocationTransform)?;

            let xbi_call = pallet::Call::execute_xbi::<T> { xbi };
            let xbi_format_msg = Xcm(vec![Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most: xbi.metadata.max_exec_cost.clone() as u64,
                call: xbi_call.encode().into(),
            }]);

            pallet_xcm::Pallet::<T>::send_xcm(
                xcm::prelude::Here,
                dest.clone(),
                xbi_format_msg.clone(),
            )
            .map_err(|_| Error::<T>::EnterFailedOnXcmSend)
        }

        fn check_xbi_instr_allowed_here(xbi_instr: XBIInstr) -> Result<(), Error<T>> {
            // todo: Expose via pallet_xbi_executor::<T>::Config
            return match xbi_instr {
                XBIInstr::CallNative { .. } => Ok(()),
                XBIInstr::CallEvm { .. } => Err(Error::<T>::XBIInstructionNotAllowedHere),
                XBIInstr::CallWasm { .. } => Err(Error::<T>::XBIInstructionNotAllowedHere),
                XBIInstr::CallCustom { .. } => Err(Error::<T>::XBIInstructionNotAllowedHere),
                XBIInstr::Transfer { .. } => Ok(()),
                XBIInstr::TransferMulti { .. } => Ok(()),
                XBIInstr::Result { .. } => Ok(()),
                XBIInstr::Notification { .. } => Ok(()),
            }
        }

        pub fn exit(checkin: XBICheckIn<T::BlockNumber>, checkout: XBICheckOut) -> Result<(), Error<T>> {
            // expect checkout to be XBI::Result
            match checkin.xbi.instr {
                XBIInstr::CallWasm { .. } => {
                    T::WASM::callback(checkin, checkout)
                },
                XBIInstr::CallCustom { .. } => {
                    T::Custom::callback(checkin, checkout)
                },
                XBIInstr::Transfer { .. } => {
                    T::Transfer::callback(checkin, checkout)
                },
                XBIInstr::TransferMulti { .. } => {
                    T::TransferMulti::callback(checkin, checkout)
                },
                XBIInstr::Result { .. } => {
                    return Err(Error::ExitUnhandled)
                },
                XBIInstr::Notification { .. } => {
                    return Err(Error::ExitUnhandled)
                },
                XBIInstr::CallNative { .. } => {
                    return Err(Error::ExitUnhandled)
                }
                XBIInstr::CallEvm { .. } => {
                    T::Evm::callback(checkin, checkout)
                }
            }
        }

        pub fn check_in_xbi(xbi: XBIFormat) -> Result<(), Error<T>> {
            let xbi_id = T::Hashing::hash(&xbi.metadata.id.encode()[..]);

            // 	Consider taking straight from Babe
            // 	type ExpectedBlockTime = ExpectedBlockTime;
            //  pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
            // Set all of the notification timers at entry after recalculating relative time to local expected block time.

            let curr_block = frame_system::Pallet::<T>::block_number();

            let delivery_timout_at_block = curr_block
                + (xbi.metadata.delivered.notification / T::ExpectedBlockTimeMs::get()).into();
            let execution_timout_at_block = curr_block
                + (xbi.metadata.executed.notification / T::ExpectedBlockTimeMs::get()).into();

            if <Self as Store>::XBICheckIns::contains_key(xbi_id) {
                return Err(Error::<T>::XBIAlreadyCheckedIn)
            }

            <Self as Store>::XBICheckIns::insert(
                xbi_id,
                XBICheckIn {
                    xbi,
                    notification_delivery_timeout: delivery_timout_at_block.into(),
                    notification_execution_timeout: execution_timout_at_block.into(),
                },
            );

            Ok(())
        }

        pub fn enter_here(xbi: XBIFormat) -> DispatchResultWithPostInfo {
            match xbi.instr {
                XBIInstr::CallNative { .. } => {
                    unimplemented!()
                },

                XBIInstr::CallEvm {
                    caller: _,
                    dest: _,
                    value: _,
                    input: _,
                    gas_limit: _,
                    max_fee_per_gas: _,
                    max_priority_fee_per_gas: _,
                    nonce: _,
                    access_list: _,
                } => {
                    unimplemented!()
                    // T::Evm::call(
                    //     caller, // origin?
                    //     caller.into(),
                    //     dest.key.into(),
                    //     input.into(),
                    //     value.into(),
                    //     gas_limit.into(),
                    //     max_fee_per_gas,
                    //     max_priority_fee_per_gas,
                    //     nonce,
                    //     access_list,
                    // )
                },
                XBIInstr::CallWasm { .. } => {},
                XBIInstr::CallCustom { .. } => {},
                XBIInstr::Transfer { .. } => {},
                XBIInstr::TransferMulti { .. } => {},
                XBIInstr::Result { .. } => {},
                XBIInstr::Notification { .. } => {},
            }

            Ok(().into())
        }
    }
}

//
//
// impl PendingRequest {
//     /// Wait for the request to complete.
//     ///
//     /// NOTE this waits for the request indefinitely.
//     pub fn wait(self) -> HttpResult {
//         match self.try_wait(None) {
//             Ok(res) => res,
//             Err(_) => panic!("Since `None` is passed we will never get a deadline error; qed"),
//         }
//     }
//
//     /// Attempts to wait for the request to finish,
//     /// but will return `Err` in case the deadline is reached.
//     pub fn try_wait(
//         self,
//         deadline: impl Into<Option<Timestamp>>,
//     ) -> Result<HttpResult, PendingRequest> {
//         Self::try_wait_all(vec![self], deadline)
//             .pop()
//             .expect("One request passed, one status received; qed")
//     }
//
//     /// Wait for all provided requests.
//     pub fn wait_all(requests: Vec<PendingRequest>) -> Vec<HttpResult> {
//         Self::try_wait_all(requests, None)
//             .into_iter()
//             .map(|r| match r {
//                 Ok(r) => r,
//                 Err(_) => panic!("Since `None` is passed we will never get a deadline error; qed"),
//             })
//             .collect()
//     }
//
//     /// Attempt to wait for all provided requests, but up to given deadline.
//     ///
//     /// Requests that are complete will resolve to an `Ok` others will return a `DeadlineReached`
//     /// error.
//     pub fn try_wait_all(
//         requests: Vec<PendingRequest>,
//         deadline: impl Into<Option<Timestamp>>,
//     ) -> Vec<Result<HttpResult, PendingRequest>> {
//         let ids = requests.iter().map(|r| r.id).collect::<Vec<_>>();
//         let statuses = sp_io::offchain::http_response_wait(&ids, deadline.into());
//
//         statuses
//             .into_iter()
//             .zip(requests.into_iter())
//             .map(|(status, req)| match status {
//                 RequestStatus::DeadlineReached => Err(req),
//                 RequestStatus::IoError => Ok(Err(Error::IoError)),
//                 RequestStatus::Invalid => Ok(Err(Error::Unknown)),
//                 RequestStatus::Finished(code) => Ok(Ok(Response::new(req.id, code))),
//             })
//             .collect()
//     }
// }
//
// /// A HTTP response.
// #[derive(RuntimeDebug)]
// pub struct Response {
//     /// Request id
//     pub id: RequestId,
//     /// Response status code
//     pub code: u16,
//     /// A collection of headers.
//     headers: Option<Headers>,
// }
