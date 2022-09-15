#![feature(inherent_associated_types)]
#![feature(associated_type_defaults)]
#![cfg_attr(not(feature = "std"), no_std)]

use xcm::latest::Xcm;

pub mod primitives;
pub mod xbi_abi;
pub mod xbi_codec;
pub mod xbi_format;
pub mod xbi_scabi;

pub use pallet::*;

pub use xcm::latest;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use crate::primitives::defi::DeFi;
    use crate::{
        primitives::{
            assets::Assets, evm::Evm, transfers::Transfers, wasm::WASM, xbi_callback::XBICallback,
            xcm::XCM,
        },
        xbi_format::*,
        xbi_scabi::Scabi,
        *,
    };
    use frame_support::pallet_prelude::*;
    use frame_system::{offchain::SendTransactionTypes, pallet_prelude::*};
    use sp_core::Hasher;
    use sp_runtime::traits::StaticLookup;
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
    pub trait Config:
        SendTransactionTypes<Call<Self>> + frame_system::Config + pallet_balances::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Call: From<Call<Self>>;

        type Transfers: Transfers<Self>;

        type Evm: Evm<Self>;

        type Assets: Assets<Self>;

        type DeFi: DeFi<Self>;

        type WASM: WASM<Self>;

        type Xcm: XCM<Self>;

        type Callback: XBICallback<Self>;

        #[pallet::constant]
        type ExpectedBlockTimeMs: Get<u32>;

        #[pallet::constant]
        type CheckInterval: Get<Self::BlockNumber>;

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
        fn on_initialize(_n: T::BlockNumber) -> Weight {
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
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AbiInstructionExecuted,
    }

    /// Errors that can occur while checking the authorship inherent.
    #[derive(Encode, sp_runtime::RuntimeDebug)]
    #[cfg_attr(feature = "std", derive(Decode))]
    pub enum InherentError {
        XbiCleanup,
    }

    impl sp_inherents::IsFatalError for InherentError {
        fn is_fatal_error(&self) -> bool {
            match self {
                InherentError::XbiCleanup => true,
            }
        }
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        EnterFailedOnXcmSend,
        EnterFailedOnMultiLocationTransform,
        ExitUnhandled,
        XBIABIFailedToCastBetweenTypesValue,
        XBIABIFailedToCastBetweenTypesAddress,
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
        NoTransferEscrowSupportedAtDest,
        NoTransferMultiEscrowSupportedAtDest,
        NoDeFiSupportedAtDest,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cleanup(origin: OriginFor<T>) -> DispatchResult {
            ensure_none(origin)?;

            // Process checkins
            let mut timeout_counter: u32 = 0;
            // Go over all unfinished Pending and Sent XBI Orders to find those that timed out
            for (xbi_id, xbi_checkin) in <XBICheckInsQueued<T>>::iter() {
                if xbi_checkin.notification_delivery_timeout
                    > frame_system::Pallet::<T>::block_number()
                {
                    if timeout_counter > T::TimeoutChecksLimit::get() {
                        break;
                    }
                    // XBI Result didn't arrive in expected time.
                    <XBICheckInsQueued<T>>::remove(xbi_id);
                    <XBICheckIns<T>>::insert(xbi_id, xbi_checkin.clone());
                    <XBICheckOutsQueued<T>>::insert(
                        xbi_id,
                        XBICheckOut::new_ignore_costs::<T>(
                            xbi_checkin.notification_delivery_timeout,
                            vec![],
                            XBICheckOutStatus::ErrorDeliveryTimeoutExceeded,
                        ),
                    );
                    timeout_counter += 1;
                }
            }
            for (xbi_id, xbi_checkin) in <XBICheckInsPending<T>>::iter() {
                if xbi_checkin.notification_execution_timeout
                    > frame_system::Pallet::<T>::block_number()
                {
                    if timeout_counter > T::TimeoutChecksLimit::get() {
                        break;
                    }
                    // XBI Result didn't arrive in expected time.
                    <XBICheckInsPending<T>>::remove(xbi_id);
                    <XBICheckIns<T>>::insert(xbi_id, xbi_checkin.clone());
                    <XBICheckOutsQueued<T>>::insert(
                        xbi_id,
                        XBICheckOut::new_ignore_costs::<T>(
                            xbi_checkin.notification_delivery_timeout,
                            vec![],
                            XBICheckOutStatus::ErrorExecutionTimeoutExceeded,
                        ),
                    );
                    timeout_counter += 1;
                }
            }

            // Process CheckIn Queue
            for (_checkin_counter, (xbi_id, xbi_checkin)) in
                (0_u32..T::CheckInLimit::get()).zip(<XBICheckInsQueued<T>>::iter())
            {
                match frame_system::offchain::SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
                    pallet::Call::enter_call::<T> {
                        checkin: xbi_checkin.clone(),
                        xbi_id,
                    }.into(),
                ) {
                    Ok(()) => { }
                    Err(e) => log::error!(
                            target: "runtime::xbi",
                            "Can't enter execution with current XBI: {:?}",
                            e,
                        ),
                }
                <XBICheckInsQueued<T>>::remove(xbi_id);
            }

            // Process Check Out Queue
            // All XBIs ready to check out (notification, results)
            for (_checkout_counter, (xbi_id, xbi_checkout)) in
                (0_u32..T::CheckOutLimit::get()).zip(<XBICheckOutsQueued<T>>::iter())
            {
                if let Err(_err) = pallet::Pallet::<T>::resolve(
                    <XBICheckIns<T>>::get(xbi_id)
                        .expect("Assume XBICheckOutsQueued is populated after XBICheckIns"),
                    xbi_checkout.clone(),
                ) {
                    log::info!("Can't exit execution with current XBI - continue and must be handled better");
                }

                <XBICheckOutsQueued<T>>::remove(xbi_id);
                <XBICheckOuts<T>>::insert(xbi_id, xbi_checkout);
            }

            Ok(())
        }

        /// Enter might be weight heavy - calls for execution into EVMs and if necessary sends the response
        /// If returns XBICheckOut means that executed instantly and the XBI order can be removed from pending checkouts
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn enter_call(
            origin: OriginFor<T>,
            checkin: XBICheckIn<T::BlockNumber>,
            xbi_id: T::Hash,
        ) -> DispatchResult {
            // let _who = ensure_signed(origin)?;

            let dest = checkin.xbi.metadata.dest_para_id;
            // If ordered execution locally via XBI : (T::MyParachainId::get(), T::MyParachainId::get())
            // Or if received XBI order of execution from remote Parachain
            if dest == T::MyParachainId::get() {
                let instant_checkout = match Self::receive(origin, checkin.xbi) {
                    Err(e) => XBICheckOut::new_ignore_costs::<T>(
                        checkin.notification_delivery_timeout,
                        e.encode(),
                        XBICheckOutStatus::ErrorFailedExecution,
                    ),
                    Ok(res) => {
                        // todo: source for the cost of XBI Dispatch - execute in credit
                        let actual_delivery_cost = 0;
                        XbiAbi::<T>::post_dispatch_info_2_xbi_checkout(
                            res,
                            checkin.notification_delivery_timeout,
                            XBICheckOutStatus::SuccessfullyExecuted,
                            actual_delivery_cost, //
                        )?
                    }
                };
                <XBICheckOutsQueued<T>>::insert(xbi_id, instant_checkout);
            } else {
                match Self::send(
                    checkin.xbi.clone(),
                    Box::new(Self::target_2_xcm_location(dest)?.into()),
                ) {
                    // Instant checkout with error
                    Err(e) => {
                        <XBICheckOutsQueued<T>>::insert(
                            xbi_id,
                            XBICheckOut::new_ignore_costs::<T>(
                                checkin.notification_delivery_timeout,
                                e.encode(),
                                XBICheckOutStatus::ErrorFailedOnXCMDispatch,
                            ),
                        );
                    }
                    // Insert pending
                    Ok(_) => {
                        <XBICheckInsPending<T>>::insert(xbi_id, checkin);
                    }
                }
            }

            Ok(())
        }

        #[pallet::weight(50_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn check_in_xbi(_origin: OriginFor<T>, xbi: XBIFormat) -> DispatchResult {
            Self::queue(xbi).map_err(|e| e.into())
        }
    }

    #[pallet::inherent]
    impl<T: Config> ProvideInherent for Pallet<T> {
        type Call = Call<T>;
        type Error = InherentError;

        const INHERENT_IDENTIFIER: InherentIdentifier = *b"xbiclean";

        fn create_inherent(_data: &InherentData) -> Option<Self::Call> {
            if frame_system::Pallet::<T>::block_number() % T::CheckInterval::get()
                == T::BlockNumber::from(0u8)
            {
                return Some(Call::cleanup {});
            }
            None
        }

        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Call::cleanup { .. })
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn target_2_xcm_location(
            target_id: u32,
        ) -> Result<xcm::latest::MultiLocation, Error<T>> {
            // Or xcm::VersionedMultiLocation::try_from(...)
            MultiLocation::try_from(Parachain(target_id))
                .map_err(|_| Error::<T>::EnterFailedOnMultiLocationTransform)
        }

        ////================================================================
        pub fn queue(xbi: XBIFormat) -> Result<(), Error<T>> {
            let xbi_id = T::Hashing::hash(&xbi.metadata.id.encode()[..]);

            if <Self as Store>::XBICheckIns::contains_key(xbi_id)
                || <Self as Store>::XBICheckInsQueued::contains_key(xbi_id)
                || <Self as Store>::XBICheckInsPending::contains_key(xbi_id)
            {
                return Err(Error::<T>::XBIAlreadyCheckedIn);
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
                    notification_delivery_timeout: delivery_timout_at_block,
                    notification_execution_timeout: execution_timout_at_block,
                },
            );

            Ok(())
        }

        pub fn send(
            xbi: XBIFormat,
            dest: Box<xcm::VersionedMultiLocation>,
        ) -> Result<(), Error<T>> {
            let dest = MultiLocation::try_from(*dest)
                .map_err(|()| Error::<T>::EnterFailedOnMultiLocationTransform)?;

            let require_weight_at_most = xbi.metadata.max_exec_cost as u64;
            let xbi_call = pallet::Call::check_in_xbi::<T> { xbi };
            let xbi_format_msg = Xcm(vec![Transact {
                origin_type: OriginKind::SovereignAccount,
                require_weight_at_most,
                call: xbi_call.encode().into(),
            }]);

            // Could have beein either Trait DI : T::Xcm::send_xcm or pallet_xcm::Pallet::<T>::send_xcm(
            T::Xcm::send_xcm(xcm::prelude::Here, dest, xbi_format_msg)
                .map_err(|_| Error::<T>::EnterFailedOnXcmSend)
        }

        ////================================================================

        /// These are functions for the receiver, this handles the format
        pub fn receive(origin: OriginFor<T>, xbi: XBIFormat) -> DispatchResultWithPostInfo {
            let caller = ensure_signed(origin.clone())?;
            match xbi.instr {
                XBIInstr::CallNative { payload: _ } => {
                    // let message_call = payload.take_decoded().map_err(|_| Error::FailedToDecode)?;
                    // let actual_weight = match message_call.dispatch(dispatch_origin) {
                    // 	Ok(post_info) => post_info.actual_weight,
                    // 	Err(error_and_info) => {
                    // 		// Not much to do with the result as it is. It's up to the parachain to ensure that the
                    // 		// message makes sense.
                    // 		error_and_info.post_info.actual_weight
                    // 	},
                    // }
                    Err(Error::<T>::XBIInstructionNotAllowedHere.into())
                }
                XBIInstr::CallEvm {
                    source,
                    target,
                    value,
                    input,
                    gas_limit,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                    nonce,
                    access_list,
                } => T::Evm::call(
                    origin,
                    source,
                    target,
                    input,
                    value,
                    gas_limit,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                    nonce,
                    access_list,
                ),
                XBIInstr::CallWasm {
                    dest,
                    value,
                    gas_limit,
                    storage_deposit_limit,
                    data,
                } => T::WASM::bare_call(
                    caller,
                    XbiAbi::<T>::address_global_2_local(dest.encode())?,
                    XbiAbi::<T>::value_global_2_local(value)?,
                    gas_limit,
                    XbiAbi::<T>::maybe_value_global_2_maybe_local(storage_deposit_limit)?,
                    data,
                    false,
                ),
                XBIInstr::CallCustom { .. } => Err(Error::<T>::XBIInstructionNotAllowedHere.into()),
                XBIInstr::Transfer { dest, value } => {
                    T::Transfers::transfer(
                        &caller,
                        &XbiAbi::<T>::address_global_2_local(dest.encode())?,
                        XbiAbi::<T>::value_global_2_local(value)?,
                        true,
                    )?;
                    Ok(().into())
                }
                XBIInstr::TransferAssets {
                    currency_id,
                    dest,
                    value,
                } => {
                    T::Assets::transfer(
                        origin,
                        currency_id,
                        <T::Lookup as StaticLookup>::unlookup(XbiAbi::<T>::address_global_2_local(
                            dest.encode(),
                        )?),
                        XbiAbi::<T>::value_global_2_local(value)?,
                    )?;
                    Ok(().into())
                }
                XBIInstr::Swap {
                    asset_out,
                    asset_in,
                    amount,
                    max_limit,
                    discount,
                } => T::DeFi::swap(
                    origin,
                    asset_out,
                    asset_in,
                    XbiAbi::<T>::value_global_2_local(amount)?,
                    XbiAbi::<T>::value_global_2_local(max_limit)?,
                    discount,
                ),
                XBIInstr::AddLiquidity {
                    asset_a,
                    asset_b,
                    amount_a,
                    amount_b_max_limit,
                } => T::DeFi::add_liquidity(
                    origin,
                    asset_a,
                    asset_b,
                    XbiAbi::<T>::value_global_2_local(amount_a)?,
                    XbiAbi::<T>::value_global_2_local(amount_b_max_limit)?,
                ),
                XBIInstr::RemoveLiquidity {
                    asset_a,
                    asset_b,
                    liquidity_amount,
                } => T::DeFi::remove_liquidity(
                    origin,
                    asset_a,
                    asset_b,
                    XbiAbi::<T>::value_global_2_local(liquidity_amount)?,
                ),
                XBIInstr::GetPrice {
                    asset_a,
                    asset_b,
                    amount,
                } => T::DeFi::get_price(
                    origin,
                    asset_a,
                    asset_b,
                    XbiAbi::<T>::value_global_2_local(amount)?,
                ),
                XBIInstr::Result { .. } => Err(Error::<T>::XBIInstructionNotAllowedHere.into()),
                XBIInstr::Unknown { .. } => Err(Error::<T>::XBIInstructionNotAllowedHere.into()),
            }
        }

        /// These are functions for the receiver, this writes back to the store and invokes its callbacks
        pub fn resolve(
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
            //     XBIInstr::TransferAssets { .. } => T::TransferAssets::callback(checkin, checkout),
            //     XBIInstr::Result { .. } => return Err(Error::ExitUnhandled),
            //     XBIInstr::Notification { .. } => return Err(Error::ExitUnhandled),
            //     XBIInstr::CallNative { .. } => return Err(Error::ExitUnhandled),
            //     XBIInstr::CallEvm { .. } => T::Evm::callback(checkin, checkout),
            // }
        }
    }
}
