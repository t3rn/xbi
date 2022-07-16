use codec::{Decode, Encode};
use sp_std::prelude::*;

use crate::xbi_format::XBIInstr;

/// Global XBI Types.
/// Could also introduce t3rn-primitives/abi but perhaps easier to rely on sp_std / global types
pub type Data = Vec<u8>;
pub type AssetId = u64; // Could also be xcm::MultiAsset
pub type Value = u128; // Could also be [u64; 2] or sp_core::U128
pub type ValueEvm = sp_core::U256; // Could also be [u64; 4]
pub type Gas = u64; // [u64; 4]
pub type AccountId32 = sp_runtime::AccountId32;
pub type AccountId20 = sp_core::H160; // Could also take it from MultiLocation::Junction::AccountKey20 { network: NetworkId, key: [u8; 20] },

pub struct XbiAbi<T> {
    pub _phantom: sp_std::marker::PhantomData<T>,
}

impl<T: crate::Config + frame_system::Config + pallet_balances::Config> XbiAbi<T> {
    pub fn value_global_2_local_unsafe(val: Value) -> T::Balance {
        Decode::decode(&mut &val.encode()[..]).unwrap()
    }

    pub fn address_global_2_local(account_bytes: Data) -> Result<T::AccountId, crate::Error<T>> {
        Decode::decode(&mut &account_bytes[..])
            .map_err(|_e| crate::Error::<T>::XBIABIFailedToCastBetweenTypesAddress)
    }

    pub fn account_local_2_global_20(account: T::AccountId) -> Result<AccountId20, crate::Error<T>> {
        Decode::decode(&mut &account.encode()[..])
            .map_err(|_e| crate::Error::<T>::XBIABIFailedToCastBetweenTypesValue)
    }

    pub fn account_local_2_global_32(account: T::AccountId) -> Result<AccountId32, crate::Error<T>> {
        Decode::decode(&mut &account.encode()[..])
            .map_err(|_e| crate::Error::<T>::XBIABIFailedToCastBetweenTypesValue)
    }

    pub fn account_global_2_local_32(account_32: AccountId32) -> Result<T::AccountId, crate::Error<T>> {
        Decode::decode(&mut &account_32.encode()[..])
            .map_err(|_e| crate::Error::<T>::XBIABIFailedToCastBetweenTypesValue)
    }

    pub fn account_global_2_local_20(account_20: AccountId20) -> Result<T::AccountId, crate::Error<T>> {
        Decode::decode(&mut &account_20.encode()[..])
            .map_err(|_e| crate::Error::<T>::XBIABIFailedToCastBetweenTypesValue)
    }

    pub fn value_global_2_local(val: Value) -> Result<T::Balance, crate::Error<T>> {
        Decode::decode(&mut &val.encode()[..])
            .map_err(|_e| crate::Error::<T>::XBIABIFailedToCastBetweenTypesValue)
    }

    pub fn value_global_2_local_evm(val: ValueEvm) -> Result<T::Balance, crate::Error<T>> {
        Decode::decode(&mut &val.encode()[..])
            .map_err(|_e| crate::Error::<T>::XBIABIFailedToCastBetweenTypesValue)
    }

    pub fn maybe_value_global_2_maybe_local(
        opt_val: Option<Value>,
    ) -> Result<Option<T::Balance>, crate::Error<T>> {
        match opt_val {
            None => Ok(None),
            Some(val) => Decode::decode(&mut &val.encode()[..])
                .map_err(|_e| crate::Error::<T>::XBIABIFailedToCastBetweenTypesValue),
        }
    }

    pub fn xbi_result_2_evm_output(_xbi_result: XBIInstr) {}

    pub fn xbi_result_2_wasm_output(_xbi_result: XBIInstr) {}
}

pub struct XbiArgsEvm {
    pub source: AccountId20,
    pub dest: AccountId20,
    pub value: Value,
    pub input: Data,
    pub gas_limit: Gas,
    pub max_fee_per_gas: ValueEvm,
    pub max_priority_fee_per_gas: Option<ValueEvm>,
    pub nonce: Option<ValueEvm>,
    pub access_list: Vec<(AccountId20, Vec<ValueEvm>)>,
}

pub struct XbiArgsWasm {
    pub dest: AccountId32,
    pub value: Value,
    pub gas_limit: Gas,
    pub storage_deposit_limit: Option<Value>,
    pub data: Data,
}
