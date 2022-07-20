use frame_support::weights::Weight;
use frame_system::pallet_prelude::OriginFor;

use sp_core::{H160, H256, U256};
use sp_std::vec::Vec;

use crate::{xbi_abi::*, xbi_format::*, Error};

pub trait Scabi<T: frame_system::Config + pallet_balances::Config> {
    fn args_evm_2_xbi_call_evm(
        origin: OriginFor<T>,
        source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>>;

    fn args_wasm_2_xbi_call_wasm(
        origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        debug: bool,
    ) -> Result<XBIInstr, crate::Error<T>>;

    fn args_evm_2_xbi_call_wasm(
        origin: OriginFor<T>,
        source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>>;

    fn args_wasm_2_xbi_evm_wasm(
        origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        debug: bool,
    ) -> Result<XBIInstr, crate::Error<T>>;

    fn xbi_call_wasm_result_2_wasm_result(
        xbi_checkout: XBICheckOut,
    ) -> Result<Vec<u8>, crate::Error<T>>;

    fn xbi_call_evm_result_2_evm_result(
        xbi_checkout: XBICheckOut,
    ) -> Result<Vec<u8>, crate::Error<T>>;

    fn xbi_call_evm_result_2_wasm_result(
        xbi_checkout: XBICheckOut,
    ) -> Result<Vec<u8>, crate::Error<T>>;

    fn xbi_call_wasm_result_2_evm_result(
        xbi_checkout: XBICheckOut,
    ) -> Result<Vec<u8>, crate::Error<T>>;
}

impl<T: crate::Config + frame_system::Config + pallet_balances::Config> Scabi<T> for XbiAbi<T> {
    fn args_evm_2_xbi_call_evm(
        _origin: OriginFor<T>,
        source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>> {
        Ok(XBIInstr::CallEvm {
            source,
            target,
            value,
            input,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
        })
    }

    fn args_wasm_2_xbi_call_wasm(
        _origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        _debug: bool,
    ) -> Result<XBIInstr, Error<T>> {
        Ok(XBIInstr::CallWasm {
            dest: XbiAbi::account_local_2_global_32(dest)?,
            value: XbiAbi::value_local_2_global(value)?,
            gas_limit,
            storage_deposit_limit: XbiAbi::maybe_value_local_2_maybe_global(storage_deposit_limit)?,
            data,
        })
    }

    /// Use in EVM precompiles / contract to auto-convert the self.call/delegate_call into args to WASM
    fn args_evm_2_xbi_call_wasm(
        _origin: OriginFor<T>,
        _source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        _max_fee_per_gas: U256,
        _max_priority_fee_per_gas: Option<U256>,
        _nonce: Option<U256>,
        _access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>> {
        Ok(XBIInstr::CallWasm {
            dest: XbiAbi::account_20_2_account_32(target, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])?,
            value: XbiAbi::value_evm_2_value(value)?,
            gas_limit,
            storage_deposit_limit: None,
            data: input,
        })
    }

    fn args_wasm_2_xbi_evm_wasm(
        origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        _debug: bool,
    ) -> Result<XBIInstr, Error<T>> {
        Ok(XBIInstr::CallEvm {
            source: XbiAbi::account_local_2_global_20(origin)?,
            target: XbiAbi::account_local_2_global_20(dest)?,
            value: XbiAbi::value_local_2_global_evm(value)?,
            input: data,
            gas_limit,
            // Hacky :( - take max_fee_per_gas from storage_deposit_limit?
            max_fee_per_gas: XbiAbi::maybe_value_local_2_global_evm(storage_deposit_limit)?,
            max_priority_fee_per_gas: None,
            nonce: None,
            access_list: vec![],
        })
    }

    fn xbi_call_wasm_result_2_wasm_result(_xbi_checkout: XBICheckOut) -> Result<Vec<u8>, Error<T>> {
        todo!()
    }

    fn xbi_call_evm_result_2_evm_result(_xbi_checkout: XBICheckOut) -> Result<Vec<u8>, Error<T>> {
        todo!()
    }

    fn xbi_call_evm_result_2_wasm_result(_xbi_checkout: XBICheckOut) -> Result<Vec<u8>, Error<T>> {
        todo!()
    }

    fn xbi_call_wasm_result_2_evm_result(_xbi_checkout: XBICheckOut) -> Result<Vec<u8>, Error<T>> {
        todo!()
    }
}
