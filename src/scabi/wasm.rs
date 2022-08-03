use crate::sabi::{Data, Gas, Sabi, SabiError, Value128};
use crate::scabi::evm::CallEvm;
use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_core::crypto::AccountId32;
use sp_std::prelude::*;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CallWasm {
    pub origin_source: AccountId32,
    pub dest: AccountId32,
    pub value: Value128,
    pub gas_limit: Gas,
    pub storage_deposit_limit: Option<Value128>,
    pub data: Data,
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

impl TryFrom<CallEvm> for CallWasm {
    type Error = SabiError;

    fn try_from(call: CallEvm) -> Result<Self, Self::Error> {
        let origin_source =
            Sabi::account_20_2_account_32(call.source, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])?;

        let dest =
            Sabi::account_20_2_account_32(call.target, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])?;

        let value = Sabi::value_256_2_value_128(call.value)?;
        let gas_limit = call.gas_limit;
        // fixme: access to storage_deposit_limit from one of the evm args
        let storage_deposit_limit = None;
        let data = call.input;
        Ok(CallWasm {
            origin_source,
            dest,
            value,
            gas_limit,
            storage_deposit_limit,
            data,
        })
    }
}
