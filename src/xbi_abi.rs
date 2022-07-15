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

    pub fn value_global_2_local(val: Value) -> Result<T::Balance, crate::Error<T>> {
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

    /// Use in EVM precompiles / contract to auto-convert the self.call/delegate_call into args to WASM
    pub fn args_evm_2_wasm(_args_evm: XbiArgsEvm) -> XbiArgsWasm {
        unimplemented!();
        // Sth of a form below: (is it safe to auto-cast value, gas limits, and cut / add null bytes to addresses?)
        // XbiArgsWasm {
        //     dest: args_evm.dest.into(),
        //     value: args_evm.value as Value,
        //     gas_limit: args_evm.gas_limit,
        //     storage_deposit_limit: None,
        //     data: args_evm.input
        // }
    }

    /// Use in WASM chain-extensions / contract to auto-convert the self.call into args to EVM contract
    pub fn args_wasm_2_evm(_args_wasm: XbiArgsWasm) -> XbiArgsEvm {
        unimplemented!();
        // Sth of a form below: (is it safe to auto-cast value, gas limits, and cut / add null bytes to addresses?)
        // XbiArgsEvm {
        //     source: args_wasm.caller,
        //     dest: args_wasm.dest.into(),
        //     value: args_wasm.value.into(),
        //     input: args_wasm.data,
        //     gas_limit: 0,
        //     max_fee_per_gas: Default::default(),
        //     max_priority_fee_per_gas: None,
        //     nonce: None,
        //     access_list: vec![]
        // }
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
