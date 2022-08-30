use crate::wasm::CallWasm;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::traits::Convert;
use sp_runtime::traits::TryMorph;
use sp_std::prelude::*;
use substrate_abi::{
    error::Error as SabiError, AccountId20, Data, Gas, SubstrateAbiConverter, Value256,
};

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub struct CallEvm {
    pub source: AccountId20, // Could use either [u8; 20] or Junction::AccountKey20
    pub target: AccountId20, // Could use either [u8; 20] or Junction::AccountKey20
    pub value: Value256,
    pub input: Data,
    pub gas_limit: Gas,
    pub max_fee_per_gas: Value256,
    pub max_priority_fee_per_gas: Option<Value256>,
    pub nonce: Option<Value256>,
    pub access_list: Vec<(AccountId20, Vec<sp_core::H256>)>, // Could use Vec<([u8; 20], Vec<[u8; 32]>)>,
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

impl TryFrom<CallWasm> for CallEvm {
    type Error = SabiError;

    fn try_from(call: CallWasm) -> Result<Self, Self::Error> {
        let source = SubstrateAbiConverter::try_morph(call.origin_source).unwrap()?;
        let target = SubstrateAbiConverter::try_morph(call.dest).unwrap()?;
        let value = SubstrateAbiConverter::convert(call.value);
        let input = call.data;
        let gas_limit = call.gas_limit;
        // fixme: hacks begin below - access to max_fee_per_gas, max_priority_fee_per_gas, nonce, access_list
        //  from one of the call_wasm params?
        let max_fee_per_gas: U256 = U256::default();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SubstrateContractAbiConverter;
    use substrate_abi::AccountId32;

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

        let call_wasm = SubstrateContractAbiConverter::try_morph(call_evm)
            .unwrap()
            .unwrap();

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
}
