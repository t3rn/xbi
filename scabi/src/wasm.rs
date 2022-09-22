use crate::evm::CallEvm;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::crypto::AccountId32;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;
use substrate_abi::TryConvert;
use substrate_abi::{error::Error as SabiError, Data, Gas, SubstrateAbiConverter, Value128};

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
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
        let origin_source = SubstrateAbiConverter::try_convert((
            call.source,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ))?;
        let dest = SubstrateAbiConverter::try_convert((
            call.target,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ))?;

        let value: u128 = SubstrateAbiConverter::convert(call.value);
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
