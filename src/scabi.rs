use crate::sabi::SabiError;
use codec::{Decode, Encode};
use sp_runtime::Either;

mod evm;
mod wasm;

// Could be optimised with Either
pub trait Convert<T: TryFrom<U, Error = SabiError>, U: TryFrom<T, Error = SabiError>> {
    type Error = SabiError;

    fn convert_t(t: T) -> Result<U, <U as TryFrom<T>>::Error> {
        U::try_from(t)
    }
    fn convert_u(u: U) -> Result<T, <T as TryFrom<U>>::Error> {
        T::try_from(u)
    }
    fn convert(e: Either<T, U>) -> Result<Either<U, T>, SabiError> {
        e.map_left(|t| U::try_from(t))
            .map_right(|t| T::try_from(t))
            .factor_err()
    }
}

pub trait SCAbi: Convert<evm::CallEvm, wasm::CallWasm> {}

pub struct ScabiConverter;

impl Convert<evm::CallEvm, wasm::CallWasm> for ScabiConverter {}

impl SCAbi for ScabiConverter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sabi::{AccountId20, AccountId32, Value256};
    use crate::scabi::evm::CallEvm;
    use crate::scabi::wasm::CallWasm;

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

        let call_wasm = ScabiConverter::convert_t(call_evm).unwrap();
        // For super generic, see here
        // let call_wasm = ScabiConverter::convert(Either::Left(call_evm)).unwrap().unwrap_left();

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
