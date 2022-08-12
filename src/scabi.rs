use codec::{Decode, Encode};

use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_std::prelude::*;

use sp_core::crypto::AccountId32;
use sp_std::vec::Vec;

use crate::sabi::*;

// Smart Contracts ABI - Scabi :)
pub trait Scabi {
    fn call_evm_2_call_wasm(call_evm: CallEvm) -> Result<CallWasm, SabiError>;

    fn call_wasm_2_call_evm(call_wasm: CallWasm) -> Result<CallEvm, SabiError>;
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CallWasm {
    origin_source: AccountId32,
    dest: AccountId32,
    value: Value128,
    gas_limit: Gas,
    storage_deposit_limit: Option<Value128>,
    data: Data,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CallEvm {
    source: AccountId20, // Could use either [u8; 20] or Junction::AccountKey20
    target: AccountId20, // Could use either [u8; 20] or Junction::AccountKey20
    value: Value256,
    input: Data,
    gas_limit: Gas,
    max_fee_per_gas: Value256,
    max_priority_fee_per_gas: Option<Value256>,
    nonce: Option<Value256>,
    access_list: Vec<(AccountId20, Vec<sp_core::H256>)>, // Could use Vec<([u8; 20], Vec<[u8; 32]>)>,
}

impl CallEvm {
    pub fn new(
        source: AccountId20,
        target: AccountId20,
        value: Value256,
        input: Data,
        gas_limit: Gas,
        max_fee_per_gas: Value256,
        max_priority_fee_per_gas: Option<Value256>,
        nonce: Option<Value256>,
        access_list: Vec<(AccountId20, Vec<sp_core::H256>)>,
    ) -> CallEvm {
        CallEvm {
            source,
            target,
            value,
            input,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
        }
    }
}

impl CallWasm {
    pub fn new(
        origin_source: AccountId32,
        dest: AccountId32,
        value: Value128,
        gas_limit: Gas,
        storage_deposit_limit: Option<Value128>,
        data: Data,
    ) -> CallWasm {
        CallWasm {
            origin_source,
            dest,
            value,
            gas_limit,
            storage_deposit_limit,
            data,
        }
    }
}

pub struct ScabiConverter {}

impl Scabi for ScabiConverter {
    fn call_evm_2_call_wasm(call_evm: CallEvm) -> Result<CallWasm, SabiError> {
        let origin_source =
            Sabi::account_20_2_account_32(call_evm.source, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])?;

        let dest =
            Sabi::account_20_2_account_32(call_evm.target, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])?;

        let value = Sabi::value_256_2_value_128(call_evm.value)?;
        let gas_limit = call_evm.gas_limit;
        // fixme: access to storage_deposit_limit from one of the evm args
        let storage_deposit_limit = None;
        let data = call_evm.input;
        Ok(CallWasm {
            origin_source,
            dest,
            value,
            gas_limit,
            storage_deposit_limit,
            data,
        })
    }

    fn call_wasm_2_call_evm(call_wasm: CallWasm) -> Result<CallEvm, SabiError> {
        let source = Sabi::account_32_2_account_20(call_wasm.origin_source)?;
        let target = Sabi::account_32_2_account_20(call_wasm.dest)?;
        let value = Sabi::value_128_2_value_256(call_wasm.value);
        let input = call_wasm.data;
        let gas_limit = call_wasm.gas_limit;
        // fixme: hacks begin below - access to max_fee_per_gas, max_priority_fee_per_gas, nonce, access_list
        //  from one of the call_wasm params?
        let max_fee_per_gas = Sabi::maybe_value_128_2_value_256(call_wasm.storage_deposit_limit);
        let max_priority_fee_per_gas = None;
        let nonce = None;
        let access_list = vec![];

        Ok(CallEvm {
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
}

#[test]
fn traverses_from_evm_args_eg_from_precompile_to_call_wasm() {
    let source = AccountId20::repeat_byte(1u8);
    let target = AccountId20::repeat_byte(2u8);
    let value = Value256::from(300);
    let input = vec![1, 2, 3];
    let gas_limit = 50u64;
    let max_fee_per_gas = Value256::from(100);
    let max_priority_fee_per_gas = None;
    let nonce = None;
    let access_list = vec![];

    let call_evm = CallEvm::new(
        source,
        target,
        value,
        input,
        gas_limit,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        nonce,
        access_list,
    );

    let call_wasm = ScabiConverter::call_evm_2_call_wasm(call_evm).unwrap();

    assert_eq!(
        call_wasm,
        CallWasm {
            origin_source: AccountId32::new([
                1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8,
                1u8, 1u8, 1u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8
            ]),
            dest: AccountId32::new([
                2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8, 2u8,
                2u8, 2u8, 2u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8
            ]),
            value: 300u128,
            gas_limit,
            storage_deposit_limit: None,
            data: vec![1u8, 2, 3],
        }
    )
}
