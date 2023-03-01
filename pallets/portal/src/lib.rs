#![allow(incomplete_features)]
#![feature(inherent_associated_types)]
#![feature(associated_type_defaults)]
#![feature(box_syntax)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::H256;
pub use substrate_abi;
pub use substrate_contracts_abi;
pub use xp_channel::{queue::QueueSignal, ChannelProgressionEmitter, Message};
pub use xp_format;

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
use xp_channel::{
    queue::ringbuffer::RingBufferTransient,
    traits::{HandlerInfo, Writable, XbiInstructionHandler},
};
use xp_format::{Status, XbiFormat, XbiInstruction, XbiMetadata, XbiResult};
use xs_channel::receiver::Receiver as XbiReceiver;
use xs_channel::sender::{frame::ReceiveCallProvider, Sender as XbiSender};

#[cfg(test)]
mod mock;
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
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::BlakeTwo256;
    use xcm::v2::SendXcm;
    use xp_channel::queue::{ringbuffer::DefaultIdx, Queue as QueueExt, QueueSignal};
    use xp_xcm::frame_traits::AssetLookup;

    /// A reexport of the Queue backed by a RingBufferTransient
    pub(crate) type Queue<Pallet> = RingBufferTransient<
        (Message, QueueSignal),
        <Pallet as Store>::BufferRange,
        <Pallet as Store>::QueueItems,
        DefaultIdx,
    >;

    /// A reexport of the Sender backed by the Queue
    pub(crate) type Sender<T> = xs_channel::sender::frame::queue_backed::Sender<
        T,
        Pallet<T>,
        Pallet<T>,
        <T as Config>::Xcm,
        <T as Config>::Call,
        Queue<Pallet<T>>,
        <T as Config>::AssetRegistry,
        u32,
    >;

    /// A reexport of the Receiver backed by the Queue
    pub(crate) type Receiver<T> = xs_channel::receiver::frame::sync::Receiver<
        T,
        Sender<T>,
        Pallet<T>,
        Queue<Pallet<T>>,
        Pallet<T>,
        Pallet<T>,
    >;

    // TODO: unify these storage items
    #[pallet::storage]
    pub type XbiRequests<T> =
        StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::Hash, XbiFormat, OptionQuery>;

    #[pallet::storage]
    pub type XbiResponses<T> =
        StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::Hash, XbiResult, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn queue_range)]
    pub(super) type BufferRange<T: Config> =
        StorageValue<_, (DefaultIdx, DefaultIdx), ValueQuery, BufferIndexDefaultValue>;

    #[pallet::type_value]
    pub(super) fn BufferIndexDefaultValue() -> (DefaultIdx, DefaultIdx) {
        (0, 0)
    }

    #[pallet::storage]
    #[pallet::getter(fn message_nonce)]
    pub(super) type MessageNonce<T: Config> =
        StorageValue<_, u32, ValueQuery, MessageNonceDefaultValue>;

    #[pallet::type_value]
    pub(super) fn MessageNonceDefaultValue() -> u32 {
        0
    }

    #[pallet::storage]
    #[pallet::getter(fn queue_item)]
    pub(super) type QueueItems<T> =
        StorageMap<_, Blake2_128Concat, DefaultIdx, (Message, QueueSignal), ValueQuery>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        // TODO: disable SendTransactionTypes<Call<Self>> for now
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        type XcmSovereignOrigin: Get<Self::AccountId>;
        /// Access to XCM functionality outside of this consensus system TODO: use XcmSender && ExecuteXcm for self execution
        type Xcm: SendXcm;
        /// Access to XCM functionality
        // type XcmExecutor: XcmExecutor<Self::Call>;
        /// Provide access to the contracts pallet or some pallet like it
        type Contracts: contracts_primitives::traits::Contracts<
            Self::AccountId,
            BalanceOf<Self>,
            Weight,
            Outcome = ContractExecResult<BalanceOf<Self>>,
        >;
        /// Provide access to the frontier evm pallet or some pallet like it
        type Evm: evm_primitives::traits::Evm<
            Self::Origin,
            Outcome = Result<(evm_primitives::CallInfo, Weight), DispatchError>,
        >;
        type Assets: Transfer<Self::AccountId>;
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Provide access to the asset registry so we can lookup, not really specific to XBI just helps us at this stage
        type AssetRegistry: AssetLookup<u32>; // TODO: this breaks for non-u32 assets

        /// Provide access to DeFI
        type DeFi: DeFi<Self>;

        // TODO: might not actually need this
        type Callback: XBICallback<Self>;

        // Queue management constants, needs revisiting TODO
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

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // `on_initialize` is executed at the beginning of the block before any extrinsic are
        // dispatched.
        //
        // This function must return the weight consumed by `on_initialize` and `on_finalize`.
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            // Anything that needs to be done at the start of the block.
            // We don't do anything here.
            // x-t3rn#4: Go over open Xtx and cancel if necessary
            0
        }

        fn on_finalize(_n: T::BlockNumber) {}

        // A runtime code run after every block and have access to extended set of APIs.
        //
        // For instance you can generate extrinsics for the upcoming produced block.
        fn offchain_worker(_n: T::BlockNumber) {}
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
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
        ResponseStored {
            hash: T::Hash,
            result: XbiResult,
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
        FailedToCastValue,
        FailedToCastAddress,
        FailedToCastHash,
        InstructionuctionNotAllowedHere,
        AlreadyCheckedIn,
        NotificationTimeoutDelivery,
        NotificationTimeoutExecution,
        CallbackUnsupported,
        EvmUnsupported,
        WasmUnsupported,
        CallNativeUnsupported,
        CallCustomUnsupported,
        TransferUnsupported,
        AssetsUnsupported,
        DefiUnsupported,
        ArithmeticErrorOverflow,
        TransferFailed,
        ResponseAlreadyStored,
    }

    /// TODO: implement benchmarks
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(50_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn send(origin: OriginFor<T>, msg: XbiFormat) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let mut msg = msg;

            // Get and increment the nonce
            let nonce = Self::message_nonce().wrapping_add(1);
            <MessageNonce<T>>::set(nonce);
            msg.metadata.enrich_id::<BlakeTwo256>(nonce, None);

            // TODO: we probably shouldnt allow send src==dest
            <Sender<T> as XbiSender<_>>::send(Message::Request(msg))
        }

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
        /// TODO: implement the queue for async channels
        #[pallet::weight(50_000 + T::DbWeight::get().writes(1) + T::DbWeight::get().reads(3))]
        pub fn process_queue(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
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
                                let key = T::Hash::decode(&mut &req.metadata.get_id()[..]).unwrap(); // TODO: remove unwrap
                                if !XbiRequests::<T>::contains_key(key) {
                                    XbiRequests::<T>::insert(key, req);
                                } else {
                                    log::warn!("Duplicate request: {:?}", req);
                                }
                            }
                        }
                        QueueSignal::XcmSendError => {
                            if let Message::Request(req) = msg {
                                let result = XbiResult {
                                    status: Status::DispatchFailed,
                                    output: vec![],
                                    witness: vec![],
                                };
                                Pallet::<T>::write((req.metadata.get_id(), result))?;
                            }
                        }
                        QueueSignal::ResponseReceived => {
                            if let Message::Response(resp, meta) = msg {
                                Pallet::<T>::write((meta.get_id(), resp))?;
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
                // TODO: handle queue parts here
                // return Some(Call::cleanup {});
            }
            None
        }

        fn is_inherent(_call: &Self::Call) -> bool {
            false
            // TODO: handle queue parts
            // matches!(call, Call::cleanup { .. })
        }
    }
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
        // TODO: implement dynamism to this
        xbi_call.push_front(200);
        xbi_call.into()
    }
}

// TODO: move to sabi
fn account_from_account32<T: Config>(
    account: &AccountId32,
) -> Result<T::AccountId, DispatchErrorWithPostInfo<PostDispatchInfo>> {
    T::AccountId::decode(&mut account.as_ref()).map_err(|_| Error::<T>::FailedToCastAddress.into())
}

// TODO: write tests
// TODO: emit errors
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
            | XbiInstruction::GetPrice { .. } => Err(Error::<T>::DefiUnsupported.into()),
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
                .map_err(|_| Error::<T>::FailedToCastValue)?;

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

// TODO: benchmarking
impl<T: Config> Writable<(H256, XbiResult)> for Pallet<T> {
    fn write(t: (H256, XbiResult)) -> sp_runtime::DispatchResult {
        let (hash, result) = t;
        let hash: T::Hash =
            Decode::decode(&mut &hash.encode()[..]).map_err(|_| Error::<T>::FailedToCastHash)?;
        if !XbiResponses::<T>::contains_key(hash) {
            XbiResponses::<T>::insert(hash, result.clone());
            Self::deposit_event(Event::<T>::ResponseStored {
                hash: hash,
                result: result.clone(),
            });
            Ok(())
        } else {
            Err(Error::<T>::ResponseAlreadyStored.into())
        }
    }
}
