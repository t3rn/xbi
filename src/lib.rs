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
        primitives::{
            assets::Assets, evm::Evm, orml::ORML, transfers::Transfers, wasm::WASM,
            xbi_callback::XBICallback,
        },
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

        type Transfers: Transfers<Self>;

        type Evm: Evm<Self>;

        type ORML: ORML<Self>;

        type Assets: Assets<Self>;

        type WASM: WASM<Self>;

        type Callback: XBICallback<Self>;

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
        fn offchain_worker(n: T::BlockNumber) {
            // We don't do anything here.
            // but we could dispatch extrinsic (transaction/unsigned/inherent) using
            // sp_io::submit_extrinsic

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
                        <XBICheckOutsQueued<T>>::insert(
                            xbi_id,
                            XBICheckOut::new(
                                xbi_checkin.notification_delivery_timeout,
                                vec![],
                                XBICheckOutStatus::ErrorDeliveryTimeout,
                            ),
                        );
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
                        <XBICheckOutsQueued<T>>::insert(
                            xbi_id,
                            XBICheckOut::new(
                                xbi_checkin.notification_delivery_timeout,
                                vec![],
                                XBICheckOutStatus::ErrorExecutionTimeout,
                            ),
                        );
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
                        },
                        // Instant check out - no pending remote XBI execution
                        Ok(Some(checkout)) => {
                            <XBICheckOutsQueued<T>>::insert(xbi_id, checkout);
                        },
                        Err(_err) => {
                            log::info!("Can't enter execution with current XBI - continue and must be handled better");
                        },
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

                    match pallet::Pallet::<T>::exit(
                        <XBICheckIns<T>>::get(xbi_id)
                            .expect("Assume XBICheckOutsQueued is populated after XBICheckIns"),
                        xbi_checkout.clone(),
                    ) {
                        // Pending remote XBI execution
                        Err(_err) => {
                            log::info!("Can't exit execution with current XBI - continue and must be handled better");
                        },
                        _ => {},
                    }
                    <XBICheckOutsQueued<T>>::remove(xbi_id.clone());
                    <XBICheckOuts<T>>::insert(xbi_id.clone(), xbi_checkout);

                    checkout_counter += 1;
                }

                timed_out_checkins
            }
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

        #[pallet::weight(50_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn check_in_xbi(_origin: OriginFor<T>, xbi: XBIFormat) -> DispatchResult {
            let xbi_id = T::Hashing::hash(&xbi.metadata.id.encode()[..]);

            if <Self as Store>::XBICheckIns::contains_key(xbi_id)
                || <Self as Store>::XBICheckInsQueued::contains_key(xbi_id)
                || <Self as Store>::XBICheckInsPending::contains_key(xbi_id)
            {
                return Err(Error::<T>::XBIAlreadyCheckedIn)
            }

            // 	Consider taking straight from Babe
            // 	type ExpectedBlockTime = ExpectedBlockTime;
            //  pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
            // Set all of the notification timers at entry after recalculating relative time to local expected block time.

            let curr_block = frame_system::Pallet::<T>::block_number();

            let delivery_timout_at_block = curr_block
                + (xbi.metadata.delivered.notification / T::ExpectedBlockTimeMs::get()).into();
            let execution_timout_at_block = curr_block
                + (xbi.metadata.executed.notification / T::ExpectedBlockTimeMs::get()).into();

            <Self as Store>::XBICheckInsQueued::insert(
                xbi_id,
                XBICheckIn {
                    xbi,
                    notification_delivery_timeout: delivery_timout_at_block.into(),
                    notification_execution_timeout: execution_timout_at_block.into(),
                },
            );

            Ok(().into())
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
                    )
                    .map_err(|e| {
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
        }

        fn enter_remote(
            xbi: XBIFormat,
            dest: Box<xcm::VersionedMultiLocation>,
        ) -> Result<(), Error<T>> {
            let dest = MultiLocation::try_from(*dest)
                .map_err(|()| Error::<T>::EnterFailedOnMultiLocationTransform)?;

            let xbi_call = pallet::Call::check_in_xbi::<T> { xbi };
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

        pub fn exit(
            checkin: XBICheckIn<T::BlockNumber>,
            checkout: XBICheckOut,
        ) -> Result<(), Error<T>> {
            // expect checkout to be XBI::Result
            T::Callback::callback(checkin, checkout);

            Ok(())
            // match checkin.xbi.instr {
            //     XBIInstr::CallWasm { .. } => T::WASM::callback(checkin, checkout),
            //     XBIInstr::CallCustom { .. } => T::Custom::callback(checkin, checkout),
            //     XBIInstr::Transfer { .. } => T::Transfer::callback(checkin, checkout),
            //     XBIInstr::TransferMulti { .. } => T::TransferMulti::callback(checkin, checkout),
            //     XBIInstr::Result { .. } => return Err(Error::ExitUnhandled),
            //     XBIInstr::Notification { .. } => return Err(Error::ExitUnhandled),
            //     XBIInstr::CallNative { .. } => return Err(Error::ExitUnhandled),
            //     XBIInstr::CallEvm { .. } => T::Evm::callback(checkin, checkout),
            // }
        }

        pub fn enter_here(xbi: XBIFormat) -> DispatchResultWithPostInfo {
            match xbi.instr {
                XBIInstr::CallNative { ref payload } => {
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
                    // T::Evm::call(
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
                    // T::WASM::call(
                    // 	caller,
                    // 	dest,
                    // 	value,
                    // 	input,
                    // )
                },
                XBIInstr::CallCustom { .. } => {
                    // T::Custom::call(
                    // 	caller,
                    // 	dest,
                    // 	value,
                    // 	input,
                    // )}
                },
                XBIInstr::Transfer {
                    ref dest,
                    ref value,
                } => {
                    // T::Transfer::call(
                    // 	caller,
                    // 	dest,
                    // 	value,
                    // 	input,
                    // )
                },
                XBIInstr::TransferMulti {
                    currency_id: _,
                    ref dest,
                    ref value,
                } => {
                    // T::Transfer::call(
                    // 	caller,
                    // 	dest,
                    // 	value,
                    // 	input,
                    // )
                },
                XBIInstr::Result { .. } => {},
                XBIInstr::Notification { .. } => {},
            }

            Ok(().into())
        }
    }
}
