use crate::sabi::{AccountId20, Data, Gas, Sabi, SabiError, Value256};
use crate::scabi::wasm::CallWasm;
use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
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
        let source = Sabi::account_32_2_account_20(call.origin_source)?;
        let target = Sabi::account_32_2_account_20(call.dest)?;
        let value = Sabi::value_128_2_value_256(call.value);
        let input = call.data;
        let gas_limit = call.gas_limit;
        // fixme: hacks begin below - access to max_fee_per_gas, max_priority_fee_per_gas, nonce, access_list
        //  from one of the call_wasm params?
        let max_fee_per_gas = Sabi::maybe_value_128_2_value_256(call.storage_deposit_limit);
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
