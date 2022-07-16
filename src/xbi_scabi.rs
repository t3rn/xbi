use frame_system::pallet_prelude::OriginFor;

use sp_core::{H160, H256, U256};
use sp_std::vec::Vec;

use crate::{xbi_abi::*, xbi_format::*};

pub trait Scabi<T: frame_system::Config> {
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

    fn args_wasm_2_xbi_call_wasm();

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

    fn args_wasm_2_xbi_evm_wasm();

    fn xbi_call_wasm_result_2_wasm_result();

    fn xbi_call_evm_result_2_evm_result();

    fn xbi_call_evm_result_2_wasm_result();

    fn xbi_call_wasm_result_2_evm_result();
}

impl<T: crate::Config + frame_system::Config + pallet_balances::Config> Scabi<T> for XbiAbi<T> {
    /// Use in EVM precompiles / contract to auto-convert the self.call/delegate_call into args to WASM
    fn args_evm_2_xbi_call_wasm(
        _origin: OriginFor<T>,
        _source: H160,
        _target: H160,
        _input: Vec<u8>,
        _value: U256,
        _gas_limit: u64,
        _max_fee_per_gas: U256,
        _max_priority_fee_per_gas: Option<U256>,
        _nonce: Option<U256>,
        _access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>> {
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

    fn args_evm_2_xbi_call_evm(
        _origin: OriginFor<T>,
        _source: H160,
        _target: H160,
        _input: Vec<u8>,
        _value: U256,
        _gas_limit: u64,
        _max_fee_per_gas: U256,
        _max_priority_fee_per_gas: Option<U256>,
        _nonce: Option<U256>,
        _access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>> {
        todo!()
    }

    fn args_wasm_2_xbi_call_wasm() {
        todo!()
    }

    fn args_wasm_2_xbi_evm_wasm() {
        todo!()
    }

    fn xbi_call_wasm_result_2_wasm_result() {
        todo!()
    }

    fn xbi_call_evm_result_2_evm_result() {
        todo!()
    }

    fn xbi_call_evm_result_2_wasm_result() {
        todo!()
    }

    fn xbi_call_wasm_result_2_evm_result() {
        todo!()
    }
}
