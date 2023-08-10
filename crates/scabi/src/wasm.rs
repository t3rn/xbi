use crate::evm::CallEvm;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::crypto::AccountId32;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;
use substrate_abi::{
    error::Error as SabiError, Data, Gas, SubstrateAbiConverter, TryConvert, Value128,
};

/// A general call to a WASM runtime
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

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::ByteArray;

    fn test_wasm() -> CallWasm {
        CallWasm::new(
            AccountId32::from([1_u8; 32]),
            AccountId32::from([2_u8; 32]),
            5000,
            200,
            None,
            "hellomynameis".encode(),
        )
    }

    #[test]
    fn try_from_evm() {
        let wasm = test_wasm();
        assert_eq!(wasm.origin_source, AccountId32::new([1_u8; 32]));
        assert_eq!(wasm.dest, AccountId32::new([2_u8; 32]));
        assert_eq!(wasm.data, "hellomynameis".encode());

        let evm = CallEvm::try_from(test_wasm()).unwrap();

        let wasm = CallWasm::try_from(evm).unwrap();
        assert_eq!(wasm.data, test_wasm().data);
        assert_eq!(wasm.value, test_wasm().value);
        assert_eq!(
            wasm.storage_deposit_limit,
            test_wasm().storage_deposit_limit
        );
        assert_eq!(wasm.gas_limit, test_wasm().gas_limit);
        // last 12 are thrown away
        assert_eq!(
            wasm.dest.as_slice()[0..20],
            test_wasm().dest.as_slice()[0..20]
        );
        // set to buffer
        assert_eq!(wasm.dest.as_slice()[20..32], [0_u8; 12]);
        assert_eq!(wasm.origin_source.as_slice()[20..32], [0_u8; 12]);
    }
}
