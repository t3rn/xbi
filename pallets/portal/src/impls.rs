use codec::{Decode, Encode};
use contracts_primitives::traits::Contracts;
use evm_primitives::traits::Evm;
use frame_support::{
    traits::{fungibles::Transfer, Currency, ExistenceRequirement},
    weights::{PostDispatchInfo, WeightToFee},
};
use frame_system::ensure_signed;
use sp_core::H256;
use sp_runtime::{
    traits::UniqueSaturatedInto, AccountId32, DispatchError, DispatchErrorWithPostInfo, Either,
};
use xp_channel::{
    traits::{HandlerInfo, Writable, XbiInstructionHandler},
    ChannelProgressionEmitter, Message,
};
use xp_format::{XbiFormat, XbiInstruction, XbiMetadata, XbiResult};
use xs_channel::sender::frame::ReceiveCallProvider;

use crate::{Config, Error, Event, Pallet, XbiResponses};

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
            crate::pallet::Call::receive::<C> { msg }.encode().into();
        // FIXME: lookup index for target from metadata, cannot be retrieved from PalletInfo
        // TODO: implement dynamism to this
        xbi_call.push_front(200);
        xbi_call.into()
    }
}

// TODO: move to sabi
pub fn account_from_account32<T: Config>(
    account: &AccountId32,
) -> Result<T::AccountId, DispatchErrorWithPostInfo<PostDispatchInfo>> {
    T::AccountId::decode(&mut account.as_ref()).map_err(|_| Error::<T>::FailedToCastAddress.into())
}

// TODO: move to sabi
pub fn account32_from_account<T: Config>(
    account: &T::AccountId,
) -> Result<AccountId32, DispatchError> {
    let account_bytes = account.encode();

    Ok(AccountId32::new(
        account_bytes
            .get(0..32)
            .and_then(|x| x.try_into().ok())
            .ok_or(Error::<T>::FailedToCastAddress)?,
    ))
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

        let result = match xbi.instr {
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
        };

        match &result {
            Ok(info) => {
                xbi.metadata.fees.push_aggregate(
                    T::FeeConversion::weight_to_fee(&info.weight).unique_saturated_into(),
                );
            }
            Err(err) => {
                xbi.metadata.fees.push_aggregate(
                    T::FeeConversion::weight_to_fee(
                        &err.post_info.actual_weight.unwrap_or_default(),
                    )
                    .unique_saturated_into(),
                );
            }
        }
        result
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
            Self::deposit_event(Event::<T>::ResponseStored { hash, result });
            Ok(())
        } else {
            Err(Error::<T>::ResponseAlreadyStored.into())
        }
    }
}
