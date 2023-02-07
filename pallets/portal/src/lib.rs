#![allow(incomplete_features)]
#![feature(inherent_associated_types)]
#![feature(associated_type_defaults)]
#![feature(box_syntax)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use substrate_abi;
pub use substrate_contracts_abi;
pub use xbi_channel_primitives::{queue::QueueSignal, ChannelProgressionEmitter, Message};
pub use xbi_format;

use codec::{Decode, Encode};
use contracts_primitives::traits::Contracts;
use evm_primitives::traits::Evm;
use frame_support::{
    traits::{fungibles::Transfer, Currency, ExistenceRequirement, Get},
    weights::PostDispatchInfo,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use sp_runtime::{traits::UniqueSaturatedInto, AccountId32, DispatchErrorWithPostInfo, Either};
use sp_std::{default::Default, prelude::*};
use xbi_channel_primitives::{
    queue::ringbuffer::RingBufferTransient,
    traits::{HandlerInfo, XbiInstructionHandler},
};
use xbi_format::{
    XbiCheckIn, XbiCheckOut, XbiCheckOutStatus, XbiFormat, XbiInstruction, XbiMetadata, XbiResult,
};
use xbi_receiver::Receiver as XbiReceiver;
use xbi_sender::{frame::ReceiveCallProvider, Sender as XbiSender};
#[cfg(test)]
mod tests;

pub mod primitives;
pub mod xbi_abi;
pub mod xbi_scabi;

t3rn_primitives::reexport_currency_types!();

#[frame_support::pallet]
pub mod pallet {
    use crate::{
        primitives::{defi::DeFi, xbi_callback::XBICallback},
        Event::{QueueEmpty, QueuePopped},
        *,
    };
    use contracts_primitives::ContractExecResult;
    use frame_support::{
        pallet_prelude::*,
        traits::{fungibles::Transfer, ReservableCurrency},
    };
    use frame_system::{offchain::SendTransactionTypes, pallet_prelude::*};
    use xbi_channel_primitives::queue::{ringbuffer::DefaultIdx, Queue as QueueExt, QueueSignal};
    use xbi_format::*;
    use xcm::v2::SendXcm;
    use xcm_primitives::frame_traits::AssetLookup;

    pub(crate) type Queue<Pallet> = RingBufferTransient<
        (Message, QueueSignal),
        <Pallet as Store>::BufferRange,
        <Pallet as Store>::QueueItems,
        DefaultIdx,
    >;
    pub(crate) type Sender<T> = xbi_sender::frame::queue_backed::Sender<
        T,
        Pallet<T>,
        Pallet<T>,
        <T as Config>::Xcm,
        <T as Config>::Call,
        Queue<Pallet<T>>,
        <T as Config>::AssetRegistry,
        u32,
    >;
    pub(crate) type Receiver<T> =
        xbi_receiver::frame::sync::Receiver<T, Sender<T>, Pallet<T>, Queue<Pallet<T>>, Pallet<T>>;

    /// Queue XBI for batch execution
    #[pallet::storage]
    pub type XbiCheckInsQueued<T> = StorageMap<
        _,
        Identity,
        <T as frame_system::Config>::Hash,
        XbiCheckIn<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;

    /// Processed XBI queue pending for execution
    #[pallet::storage]
    pub type XbiCheckInsPending<T> = StorageMap<
        _,
        Identity,
        <T as frame_system::Config>::Hash,
        XbiCheckIn<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// XBI called for execution
    pub type XbiCheckIns<T> = StorageMap<
        _,
        Identity,
        <T as frame_system::Config>::Hash,
        XbiCheckIn<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;

    #[pallet::storage]
    /// Lifecycle: If executed: XbiCheckInsPending -> XbiCheckIns -> XbiCheckOutsQueued
    /// Lifecycle: If not executed: XbiCheckInsPending -> XbiCheckOutsQueued
    pub type XbiCheckOutsQueued<T> =
        StorageMap<_, Identity, <T as frame_system::Config>::Hash, XbiCheckOut, OptionQuery>;

    #[pallet::storage]
    /// XBI Results of execution on local (here) Parachain
    pub type XbiCheckOuts<T> =
        StorageMap<_, Identity, <T as frame_system::Config>::Hash, XbiCheckOut, OptionQuery>;

    // ==================== New storage items ==========================
    #[pallet::storage]
    pub type XbiRequests<T> =
        StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::Hash, XbiFormat, OptionQuery>;

    #[pallet::storage]
    pub type XbiResponses<T> =
        StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::Hash, XbiResult, OptionQuery>;

    /// Timesheet information, utilised by the queue to determine the next step
    #[pallet::storage]
    pub type XbiTimesheet<T> = StorageMap<
        _,
        Blake2_128Concat,
        <T as frame_system::Config>::Hash,
        XbiTimeSheet<<T as frame_system::Config>::BlockNumber>,
        OptionQuery,
    >;

    /// What is the job of the queue?
    ///
    /// - Take requests and replies:
    ///     - update the status
    ///     - update the timesheet
    ///     - write any results
    ///
    /// - Optionally, in the future:
    ///     - send requests/responses, used as a gate to XCM
    ///
    ///

    #[pallet::storage]
    #[pallet::getter(fn queue_range)]
    pub(super) type BufferRange<T: Config> =
        StorageValue<_, (DefaultIdx, DefaultIdx), ValueQuery, BufferIndexDefaultValue>;

    #[pallet::type_value]
    pub(super) fn BufferIndexDefaultValue() -> (DefaultIdx, DefaultIdx) {
        (0, 0)
    }

    #[pallet::storage]
    #[pallet::getter(fn queue_item)]
    pub(super) type QueueItems<T> =
        StorageMap<_, Blake2_128Concat, DefaultIdx, (Message, QueueSignal), ValueQuery>;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: SendTransactionTypes<Call<Self>> + frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Call: From<Call<Self>>;

        /// Access to XCM functionality outside of this consensus system TODO: use XcmSender && ExecuteXcm like xtokens
        type Xcm: SendXcm;

        /// Access to XCM functionality
        // type XcmExecutor: XcmExecutor<Self::Call>;

        type XcmSovereignOrigin: Get<Self::AccountId>;

        type Contracts: contracts_primitives::traits::Contracts<
            Self::AccountId,
            BalanceOf<Self>,
            Weight,
            Outcome = ContractExecResult<BalanceOf<Self>>,
        >;

        type Evm: evm_primitives::traits::Evm<
            Self::Origin,
            Outcome = Result<(evm_primitives::CallInfo, Weight), DispatchError>,
        >;

        type Assets: Transfer<Self::AccountId>;

        type AssetRegistry: AssetLookup<u32>; // TODO: this breaks for non-u32 assets

        type DeFi: DeFi<Self>;

        type Callback: XBICallback<Self>;

        type Currency: ReservableCurrency<Self::AccountId>;

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
        type ParachainId: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // TODO: ensure at-most-once enqueue if using an OCW to handle queue
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
        /// TODO: remove after testing
        XbiMessageReceived {
            request: Option<XbiFormat>,
            response: Option<XbiResult>,
        },
        XbiMessageSent {
            msg: Message,
        },
        XbiRequestHandled {
            result: XbiResult,
            metadata: XbiMetadata,
            weight: Weight,
        },
        XbiInstructionHandled {
            msg: XbiFormat,
            weight: Weight,
        },
        QueueEmpty,
        QueuePopped {
            signal: QueueSignal,
            msg: Message,
        },
        QueuePushed {
            signal: QueueSignal,
            msg: Message,
        },
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
        XbiInstructionuctionNotAllowedHere,
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
        ArithmeticErrorOverflow,
        NoCallNativeSupportedAtDest,
        NoCallCustomSupportedAtDest,
        TransferFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cleanup(origin: OriginFor<T>) -> DispatchResult {
            ensure_none(origin)?;

            // Process checkins
            let mut timeout_counter: u32 = 0;
            // Go over all unfinished Pending and Sent XBI Orders to find those that timed out
            for (xbi_id, xbi_checkin) in <XbiCheckInsQueued<T>>::iter() {
                if xbi_checkin.notification_delivery_timeout
                    > frame_system::Pallet::<T>::block_number()
                {
                    if timeout_counter > T::TimeoutChecksLimit::get() {
                        break;
                    }
                    // XBI Result didn't arrive in expected time.
                    <XbiCheckInsQueued<T>>::remove(xbi_id);
                    <XbiCheckIns<T>>::insert(xbi_id, xbi_checkin.clone());
                    <XbiCheckOutsQueued<T>>::insert(
                        xbi_id,
                        XbiCheckOut::new_ignore_costs::<T>(
                            xbi_id,
                            xbi_checkin.notification_delivery_timeout,
                            vec![],
                            XbiCheckOutStatus::ErrorDeliveryTimeoutExceeded,
                        ),
                    );
                    timeout_counter += 1;
                }
            }
            for (xbi_id, xbi_checkin) in <XbiCheckInsPending<T>>::iter() {
                if xbi_checkin.notification_execution_timeout
                    > frame_system::Pallet::<T>::block_number()
                {
                    if timeout_counter > T::TimeoutChecksLimit::get() {
                        break;
                    }
                    // XBI Result didn't arrive in expected time.
                    <XbiCheckInsPending<T>>::remove(xbi_id);
                    <XbiCheckIns<T>>::insert(xbi_id, xbi_checkin.clone());
                    <XbiCheckOutsQueued<T>>::insert(
                        xbi_id,
                        XbiCheckOut::new_ignore_costs::<T>(
                            xbi_id,
                            xbi_checkin.notification_delivery_timeout,
                            vec![],
                            XbiCheckOutStatus::ErrorExecutionTimeoutExceeded,
                        ),
                    );
                    timeout_counter += 1;
                }
            }

            // Process CheckIn Queue
            for (_checkin_counter, (xbi_id, xbi_checkin)) in
                (0_u32..T::CheckInLimit::get()).zip(<XbiCheckInsQueued<T>>::iter())
            {
                match frame_system::offchain::SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(
                    pallet::Call::send::<T> {
                        msg: Message::Request(xbi_checkin.xbi)
                    }.into(),
                ) {
                    Ok(()) => { }
                    Err(e) => log::error!(
                            target: "runtime::xbi",
                            "Can't enter execution with current XBI: {:?}",
                            e,
                        ),
                }
                <XbiCheckInsQueued<T>>::remove(xbi_id);
            }

            // // Process Check Out Queue
            // // All XBIs ready to check out (notification, results)
            // for (_checkout_counter, (xbi_id, xbi_checkout)) in
            //     (0_u32..T::CheckOutLimit::get()).zip(<XbiCheckOutsQueued<T>>::iter())
            // {
            //     if let Err(_err) = pallet::Pallet::<T>::resolve(
            //         <XbiCheckIns<T>>::get(xbi_id)
            //             .expect("Assume XbiCheckOutsQueued is populated after XbiCheckIns"),
            //         xbi_checkout.clone(),
            //     ) {
            //         log::info!("Can't exit execution with current XBI - continue and must be handled better");
            //     }
            //
            //     <XbiCheckOutsQueued<T>>::remove(xbi_id);
            //     <XbiCheckOuts<T>>::insert(xbi_id, xbi_checkout);
            // }

            Ok(())
        }

        // /// Enter might be weight heavy - calls for execution into EVMs and if necessary sends the response
        // /// If returns XbiCheckOut means that executed instantly and the XBI order can be removed from pending checkouts
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn enter_call(
        //     origin: OriginFor<T>,
        //     checkin: XbiCheckIn<T::BlockNumber>,
        //     xbi_id: T::Hash,
        // ) -> DispatchResult {
        //     let _who = ensure_signed(origin.clone())?;
        //
        //     let dest = checkin.xbi.metadata.dest_para_id;
        //
        //     // TODO: bake this in pre-send
        //     // If ordered execution locally via XBI : (T::MyParachainId::get(), T::MyParachainId::get())
        //     // Or if received XBI order of execution from remote Parachain
        //     if dest == T::ParachainId::get() {
        //         // TODO: receiver must take checkouts & checkins if both handler and results
        //         let instant_checkout = match Self::channel_receive(origin, checkin.clone()) {
        //             Err(e) => XbiCheckOut::new_ignore_costs::<T>(
        //                 xbi_id,
        //                 checkin.notification_delivery_timeout,
        //                 e.encode(),
        //                 XbiCheckOutStatus::ErrorFailedExecution,
        //             ),
        //             Ok(res) => {
        //                 // todo: source for the cost of XBI Dispatch - execute in credit
        //                 let actual_delivery_cost = 0;
        //                 XbiAbi::<T>::post_dispatch_info_2_xbi_checkout(
        //                     xbi_id,
        //                     res,
        //                     checkin.notification_delivery_timeout,
        //                     XbiCheckOutStatus::SuccessfullyExecuted,
        //                     actual_delivery_cost, //
        //                 )?
        //             }
        //         };
        //         <XbiCheckOutsQueued<T>>::insert(xbi_id, instant_checkout);
        //     } else {
        //         let msg = Message::Request(checkin.xbi.clone());
        //         <Sender<T> as XbiSender<_>>::send(msg);
        //     }
        //
        //     Ok(())
        // }

        // TODO: users should not be able to provide the ID, it should have a nonce appended and we hash it
        /// TODO: implement benchmarks
        /// This will send an xbi message
        #[pallet::weight(50_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn send(origin: OriginFor<T>, msg: Message) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            // TODO: here we will reinstate instant_checkout, check if sender is same as receiver
            // and store the result if needed, don't let this pass through to the channel
            <Sender<T> as XbiSender<_>>::send(msg)
        }

        /// TODO: implement benchmarks
        /// This receive api is called by the sender on the source parachain and needs to exist for
        /// the handler to be able to invoke
        ///
        /// There are additional ways this can be called:
        ///     - expose the same interface but allow some pathway to it: Contracts::call {..}
        ///     - expose a way to call a pallet method
        #[pallet::weight(50_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn receive(origin: OriginFor<T>, msg: Message) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin.clone())?;
            <Receiver<T> as XbiReceiver>::receive(origin, msg)
        }

        /// TODO: implement benchmarks
        /// TODO: do stuff with this
        #[pallet::weight(50_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn process_queue(origin: OriginFor<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let mut queue = <Queue<Pallet<T>>>::default();
            if queue.is_empty() {
                Self::deposit_event(QueueEmpty);
            } else {
                while let Some((msg, signal)) = queue.pop() {
                    Self::deposit_event(QueuePopped {
                        signal: signal.clone(),
                        msg: msg.clone(),
                    });
                    match signal {
                        QueueSignal::PendingResponse => {
                            if let Message::Request(req) = msg {
                                let key = T::Hash::decode(&mut &req.metadata.id[..]).unwrap(); // TODO: remove unwrap
                                if !XbiRequests::<T>::contains_key(key) {
                                    XbiRequests::<T>::insert(key, req);
                                } else {
                                    log::warn!("Duplicate request: {:?}", req);
                                }
                            }
                        }
                        QueueSignal::XcmSendError => {
                            if let Message::Request(req) = msg {
                                let key = T::Hash::decode(&mut &req.metadata.id[..]).unwrap(); // TODO: remove unwrap

                                let result = XbiResult {
                                    id: req.metadata.id.clone().as_bytes().to_vec(),
                                    status: XbiCheckOutStatus::ErrorFailedOnXCMDispatch,
                                    output: vec![],
                                    witness: vec![],
                                };
                                if !XbiResponses::<T>::contains_key(key) {
                                    XbiResponses::<T>::insert(key, result);
                                } else {
                                    log::warn!("Duplicate response: {:?}", result);
                                }
                            }
                        }
                        QueueSignal::ResponseReceived => {
                            if let Message::Response(resp, meta) = msg {
                                let key = T::Hash::decode(&mut &meta.id[..]).unwrap(); // TODO: remove unwrap

                                // TODO: update meta, asserting exists
                                // TODO: write response
                                if !XbiResponses::<T>::contains_key(key) {
                                    XbiResponses::<T>::insert(key, resp);
                                } else {
                                    log::warn!("Duplicate response: {:?}", resp);
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
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

    // TODO: not used, serves as reminder of old functionality
    // impl<T: Config> Pallet<T> {
    //     ////================================================================
    //     pub fn queue(xbi: XbiFormat) -> Result<(), Error<T>> {
    //         let xbi_id = xbi.metadata.id::<T::Hashing>();
    //
    //         if <Self as Store>::XbiCheckIns::contains_key(xbi_id)
    //             || <Self as Store>::XbiCheckInsQueued::contains_key(xbi_id)
    //             || <Self as Store>::XbiCheckInsPending::contains_key(xbi_id)
    //         {
    //             return Err(Error::<T>::XBIAlreadyCheckedIn);
    //         }
    //
    //         // 	Consider taking straight from Babe
    //         // 	type ExpectedBlockTime = ExpectedBlockTime;
    //         //  pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
    //         // Set all of the notification timers at entry after recalculating relative time to local expected block time.
    //         let curr_block = frame_system::Pallet::<T>::block_number();
    //
    //         let delivery_timout_at_block = curr_block
    //             + (xbi.metadata.timeouts.delivered.notification / T::ExpectedBlockTimeMs::get())
    //                 .into();
    //         let execution_timout_at_block = curr_block
    //             + (xbi.metadata.timeouts.executed.notification / T::ExpectedBlockTimeMs::get())
    //                 .into();
    //
    //         <Self as Store>::XbiCheckInsQueued::insert(
    //             xbi_id,
    //             XbiCheckIn {
    //                 xbi,
    //                 notification_delivery_timeout: delivery_timout_at_block,
    //                 notification_execution_timeout: execution_timout_at_block,
    //             },
    //         );
    //
    //         Ok(())
    //     }
    //
    //     ////================================================================
    //
    //     /// These are functions for the receiver, this handles the format
    //     pub fn channel_receive(
    //         origin: OriginFor<T>,
    //         checkin: XbiCheckIn<T::BlockNumber>,
    //     ) -> DispatchResultWithPostInfo {
    //         let _who = ensure_signed(origin.clone())?;
    //
    //         let dest = checkin.xbi.metadata.src_para_id;
    //         let xbi_id = checkin.xbi.metadata.id::<T::Hashing>();
    //
    //         match Self::handle_instruction(origin.clone(), checkin.xbi.clone()) {
    //             Ok(info) => {
    //                 // If ordered execution locally via XBI : (T::MyParachainId::get(), T::MyParachainId::get())
    //                 // Or if received XBI order of execution from remote Parachain
    //                 if dest == T::ParachainId::get() {
    //                     let actual_delivery_cost = 0;
    //                     // todo: source for the  of XBI Dispatch - execute in credit
    //                     let instant_checkout = XbiAbi::<T>::post_dispatch_info_2_xbi_checkout(
    //                         xbi_id,
    //                         info,
    //                         checkin.notification_delivery_timeout,
    //                         XbiCheckOutStatus::SuccessfullyExecuted,
    //                         actual_delivery_cost,
    //                     )?;
    //                     <XbiCheckOutsQueued<T>>::insert(xbi_id, instant_checkout);
    //                 } else {
    //                     // <Self as Sender<_>>::send((
    //                     //     origin,
    //                     //     dest,
    //                     //     Message::Request(checkin.xbi.clone()),
    //                     // ));
    //                 }
    //                 Ok(info)
    //             }
    //             Err(e) => {
    //                 let status = XbiCheckOutStatus::ErrorFailedExecution;
    //                 let output = e.encode();
    //                 let checkout = XbiCheckOut::new_ignore_costs::<T>(
    //                     xbi_id,
    //                     checkin.notification_delivery_timeout,
    //                     output.clone(),
    //                     status.clone(),
    //                 );
    //                 <XbiCheckOutsQueued<T>>::insert(xbi_id, checkout);
    //                 // <Self as Sender<_>>::send(Message::Response(XbiResult {
    //                 //     id: xbi_id.encode(),
    //                 //     status: status,
    //                 //     output: output,
    //                 //     witness: vec![],
    //                 // }));
    //                 Err(e)
    //             }
    //         }
    //     }
    //
    //     /// Handle the instruction
    //     pub fn handle_instruction(
    //         origin: OriginFor<T>,
    //         xbi: XbiFormat,
    //     ) -> DispatchResultWithPostInfo {
    //         let caller = ensure_signed(origin.clone())?;
    //         match xbi.instr {
    //             XbiInstruction::CallNative { payload: _ } => {
    //                 // let message_call = payload.take_decoded().map_err(|_| Error::FailedToDecode)?;
    //                 // let actual_weight = match message_call.dispatch(dispatch_origin) {
    //                 // 	Ok(post_info) => post_info.actual_weight,
    //                 // 	Err(error_and_info) => {
    //                 // 		// Not much to do with the result as it is. It's up to the parachain to ensure that the
    //                 // 		// message makes sense.
    //                 // 		error_and_info.post_info.actual_weight
    //                 // 	},
    //                 // }
    //                 Err(Error::<T>::XbiInstructionuctionNotAllowedHere.into())
    //             }
    //             XbiInstruction::CallEvm {
    //                 source,
    //                 target,
    //                 value,
    //                 input,
    //                 gas_limit,
    //                 max_fee_per_gas,
    //                 max_priority_fee_per_gas,
    //                 nonce,
    //                 access_list,
    //             } => T::Evm::call(
    //                 origin,
    //                 source,
    //                 target,
    //                 input,
    //                 value,
    //                 gas_limit,
    //                 max_fee_per_gas,
    //                 max_priority_fee_per_gas,
    //                 nonce,
    //                 access_list,
    //             ),
    //             XbiInstruction::CallWasm {
    //                 dest,
    //                 value,
    //                 gas_limit,
    //                 storage_deposit_limit,
    //                 data,
    //             } => T::WASM::bare_call(
    //                 caller,
    //                 XbiAbi::<T>::address_global_2_local(dest.encode())?,
    //                 XbiAbi::<T>::value_global_2_local(value)?,
    //                 gas_limit,
    //                 XbiAbi::<T>::maybe_value_global_2_maybe_local(storage_deposit_limit)?,
    //                 data,
    //                 false,
    //             ),
    //             XbiInstruction::CallCustom { .. } => {
    //                 Err(Error::<T>::XbiInstructionuctionNotAllowedHere.into())
    //             }
    //             XbiInstruction::Transfer { dest, value } => {
    //                 T::Transfers::transfer(
    //                     &caller,
    //                     &XbiAbi::<T>::address_global_2_local(dest.encode())?,
    //                     XbiAbi::<T>::value_global_2_local(value)?,
    //                     true,
    //                 )?;
    //                 Ok(().into())
    //             }
    //             XbiInstruction::TransferAssets {
    //                 currency_id,
    //                 dest,
    //                 value,
    //             } => {
    //                 // T::Assets::transfer(
    //                 //     origin,
    //                 //     currency_id,
    //                 //     <T::Lookup as StaticLookup>::unlookup(XbiAbi::<T>::address_global_2_local(
    //                 //         dest.encode(),
    //                 //     )?),
    //                 //     XbiAbi::<T>::value_global_2_local(value)?,
    //                 // )?;
    //                 Ok(().into())
    //             }
    //             XbiInstruction::Swap {
    //                 asset_out,
    //                 asset_in,
    //                 amount,
    //                 max_limit,
    //                 discount,
    //             } => {
    //                 // T::DeFi::swap(
    //                 //     origin,
    //                 //     asset_out,
    //                 //     asset_in,
    //                 //     XbiAbi::<T>::value_global_2_local(amount)?,
    //                 //     XbiAbi::<T>::value_global_2_local(max_limit)?,
    //                 //     discount,
    //                 // )?;
    //                 Ok(().into())
    //             }
    //             XbiInstruction::AddLiquidity {
    //                 asset_a,
    //                 asset_b,
    //                 amount_a,
    //                 amount_b_max_limit,
    //             } => {
    //                 // T::DeFi::add_liquidity(
    //                 //     origin,
    //                 //     asset_a,
    //                 //     asset_b,
    //                 //     XbiAbi::<T>::value_global_2_local(amount_a)?,
    //                 //     XbiAbi::<T>::value_global_2_local(amount_b_max_limit)?,
    //                 // );
    //                 Ok(().into())
    //             }
    //             XbiInstruction::RemoveLiquidity {
    //                 asset_a,
    //                 asset_b,
    //                 liquidity_amount,
    //             } => {
    //                 // T::DeFi::remove_liquidity(
    //                 //     origin,
    //                 //     asset_a,
    //                 //     asset_b,
    //                 //     XbiAbi::<T>::value_global_2_local(liquidity_amount)?,
    //                 // );
    //                 Ok(().into())
    //             }
    //             XbiInstruction::GetPrice {
    //                 asset_a,
    //                 asset_b,
    //                 amount,
    //             } => {
    //                 // T::DeFi::get_price(
    //                 //     origin,
    //                 //     asset_a,
    //                 //     asset_b,
    //                 //     XbiAbi::<T>::value_global_2_local(amount)?,
    //                 // );
    //                 Ok(().into())
    //             }
    //             XbiInstruction::Result { .. } => {
    //                 Err(Error::<T>::XbiInstructionuctionNotAllowedHere.into())
    //             }
    //             XbiInstruction::Unknown { .. } => {
    //                 Err(Error::<T>::XbiInstructionuctionNotAllowedHere.into())
    //             }
    //         }
    //     }
    //
    //     /// These are functions for the receiver, this writes back to the store and invokes its callbacks
    //     pub fn resolve(
    //         checkin: XbiCheckIn<T::BlockNumber>,
    //         checkout: XbiCheckOut,
    //     ) -> Result<(), Error<T>> {
    //         // expect checkout to be XBI::Result
    //         T::Callback::callback(checkin, checkout);
    //
    //         Ok(())
    //         // match checkin.xbi.instr {
    //         //     XbiInstruction::CallWasm { .. } => T::WASM::callback(checkin, checkout),
    //         //     XbiInstruction::CallCustom { .. } => T::Custom::callback(checkin, checkout),
    //         //     XbiInstruction::Transfer { .. } => T::Transfer::callback(checkin, checkout),
    //         //     XbiInstruction::TransferAssets { .. } => T::TransferAssets::callback(checkin, checkout),
    //         //     XbiInstruction::Result { .. } => return Err(Error::ExitUnhandled),
    //         //     XbiInstruction::Notification { .. } => return Err(Error::ExitUnhandled),
    //         //     XbiInstruction::CallNative { .. } => return Err(Error::ExitUnhandled),
    //         //     XbiInstruction::CallEvm { .. } => T::Evm::callback(checkin, checkout),
    //         // }
    //     }
    // }
}

impl<T: Config> ChannelProgressionEmitter for Pallet<T> {
    fn emit_instruction_handled(msg: &XbiFormat, weight: &u64) {
        use crate::Event::*;
        Self::deposit_event(XbiInstructionHandled {
            msg: msg.clone(),
            weight: *weight,
        })
    }

    fn emit_received(msg: Either<&XbiFormat, &XbiResult>) {
        use crate::Event::*;
        match msg {
            Either::Left(x) => {
                Self::deposit_event(XbiMessageReceived {
                    request: Some(x.clone()),
                    response: None,
                });
            }
            Either::Right(x) => {
                Self::deposit_event(XbiMessageReceived {
                    request: None,
                    response: Some(x.clone()),
                });
            }
        }
    }

    fn emit_request_handled(result: &XbiResult, metadata: &XbiMetadata, weight: &u64) {
        use crate::Event::*;
        Self::deposit_event(XbiRequestHandled {
            result: result.clone(),
            metadata: metadata.clone(),
            weight: *weight,
        });
    }

    fn emit_sent(msg: Message) {
        use crate::Event::*;
        Self::deposit_event(XbiMessageSent { msg });
    }
}

impl<C: Config> ReceiveCallProvider for Pallet<C> {
    fn provide<T: Into<Message>>(t: T) -> Vec<u8> {
        let msg = t.into();
        let mut xbi_call: sp_std::collections::vec_deque::VecDeque<u8> =
            pallet::Call::receive::<C> { msg }.encode().into();
        // FIXME: lookup index for target from metadata, cannot be retrieved from PalletInfo
        xbi_call.push_front(200);
        xbi_call.into()
    }
}

// TODO: move to sabi
fn account_from_account32<T: Config>(
    account: &AccountId32,
) -> Result<T::AccountId, DispatchErrorWithPostInfo<PostDispatchInfo>> {
    T::AccountId::decode(&mut account.as_ref())
        .map_err(|_| Error::<T>::XBIABIFailedToCastBetweenTypesAddress.into())
}

// TODO: write tests
impl<T: Config> XbiInstructionHandler<T::Origin> for Pallet<T> {
    fn handle(
        origin: &T::Origin,
        xbi: &mut XbiFormat,
    ) -> Result<
        HandlerInfo<frame_support::weights::Weight>,
        DispatchErrorWithPostInfo<PostDispatchInfo>,
    > {
        let caller = ensure_signed(origin.clone())?;

        log::debug!(target: "xbi", "Handling instruction for caller {:?} and message {:?}", caller, xbi);

        match xbi.instr {
            XbiInstruction::Transfer { ref dest, value } => T::Currency::transfer(
                &caller,
                &account_from_account32::<T>(dest)?,
                value.unique_saturated_into(),
                ExistenceRequirement::AllowDeath,
            )
            .map(|_| Default::default())
            .map_err(|_| Error::<T>::TransferFailed.into()),
            XbiInstruction::CallWasm {
                ref dest,
                value,
                gas_limit,
                storage_deposit_limit,
                ref data,
            } => {
                let contract_result = T::Contracts::call(
                    caller,
                    account_from_account32::<T>(dest)?,
                    value.unique_saturated_into(),
                    gas_limit,
                    storage_deposit_limit.map(UniqueSaturatedInto::unique_saturated_into),
                    data.clone(),
                    false, // ALWAYS FALSE, could panic the runtime unless over rpc
                );
                contract_result
                    .result
                    .map(|r| HandlerInfo {
                        output: r.data.0,
                        weight: contract_result.gas_consumed,
                    })
                    .map_err(|e| DispatchErrorWithPostInfo {
                        post_info: PostDispatchInfo {
                            actual_weight: Some(contract_result.gas_consumed),
                            pays_fee: Default::default(),
                        },
                        error: e,
                    })
            }
            XbiInstruction::CallEvm {
                source,
                target,
                value,
                ref input,
                gas_limit,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                ref access_list,
            } => {
                let evm_result = T::Evm::call(
                    origin.clone(),
                    source,
                    target,
                    input.clone(),
                    value,
                    gas_limit,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                    nonce,
                    access_list.clone(),
                );
                let weight = evm_result.clone().map(|(_, weight)| weight);

                evm_result
                    .map(|(x, weight)| HandlerInfo {
                        output: x.value,
                        weight,
                    })
                    .map_err(|e| DispatchErrorWithPostInfo {
                        post_info: PostDispatchInfo {
                            actual_weight: weight.ok(),
                            pays_fee: Default::default(),
                        },
                        error: e,
                    })
            }
            XbiInstruction::Swap { .. }
            | XbiInstruction::AddLiquidity { .. }
            | XbiInstruction::RemoveLiquidity { .. }
            | XbiInstruction::GetPrice { .. } => Err(Error::<T>::NoDeFiSupportedAtDest.into()),
            XbiInstruction::TransferAssets {
                currency_id,
                ref dest,
                value,
            } => {
                let keep_alive = true;

                let currency_id = <T::Assets as frame_support::traits::fungibles::Inspect<
                    T::AccountId,
                >>::AssetId::decode(
                    &mut &currency_id.encode()[..]
                )
                .map_err(|_| Error::<T>::XBIABIFailedToCastBetweenTypesValue)?;

                // TODO: have an assertion that the destination actually was updated
                T::Assets::transfer(
                    currency_id,
                    &caller,
                    &account_from_account32::<T>(dest)?,
                    value.unique_saturated_into(),
                    keep_alive,
                )
                .map(|_| Default::default())
                .map_err(|_| Error::<T>::TransferFailed.into())
            }
            ref x => {
                log::debug!(target: "xbi", "unhandled instruction: {:?}", x);
                Ok(Default::default())
            }
        }
    }
}
