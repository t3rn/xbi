use codec::{Decode, Encode};
use error::Error;
use sp_core::U256;
use sp_runtime::traits::{Convert, TryMorph};
use sp_std::prelude::*;
use std::marker::PhantomData;

/// Global XBI Types.
pub type Data = Vec<u8>;
pub type AssetId = u64; // Could also be xcm::MultiAsset
pub type Gas = u64; // [u64; 4]
pub type AccountId32 = sp_runtime::AccountId32;
pub type AccountId20 = sp_core::H160; // Could also take it from MultiLocation::Junction::AccountKey20 { network: NetworkId, key: [u8; 20] },

pub type Value32 = u32;
pub type Value64 = u64;
pub type Value128 = u128;
pub type Value256 = U256;

pub mod error;

pub enum ValueLen {
    U32,
    U64,
    U128,
    U256,
}

pub trait SubstrateAbi:
    TryMorph<(AccountId20, [u8; 12]), Outcome = Result<AccountId32, Error>>
    + TryMorph<AccountId32, Outcome = Result<AccountId20, Error>>
    + Convert<u32, u64>
    + Convert<u32, u128>
    + Convert<u32, U256>
    + Convert<u64, u32>
    + Convert<u64, u128>
    + Convert<u64, U256>
    + Convert<u128, u32>
    + Convert<u128, u64>
    + Convert<u128, U256>
{
}
pub struct SubstrateAbiConverter;

impl TryMorph<(AccountId20, [u8; 12])> for SubstrateAbiConverter {
    type Outcome = Result<AccountId32, Error>;

    fn try_morph(value: (AccountId20, [u8; 12])) -> Result<Self::Outcome, ()> {
        let mut dest_bytes: Vec<u8> = vec![];
        dest_bytes.append(&mut value.0.encode());
        dest_bytes.append(&mut value.1.encode());

        let result: Self::Outcome = Decode::decode(&mut &dest_bytes.as_slice()[..])
            .map_err(|_e| Error::FailedToCastBetweenTypesValue);
        Ok(result)
    }
}

impl TryMorph<AccountId32> for SubstrateAbiConverter {
    type Outcome = Result<AccountId20, Error>;

    fn try_morph(account_32: AccountId32) -> Result<Self::Outcome, ()> {
        let mut dest_bytes: Vec<u8> = vec![];
        let account_32_encoded = account_32.encode(); // hmm doesnt this add Len? FIXME: use as_ref

        for &byte_of_account in account_32_encoded.iter().take(20) {
            dest_bytes.push(byte_of_account);
        }

        let result: Result<AccountId20, Error> = Decode::decode(&mut &dest_bytes.as_slice()[..])
            .map_err(|_e| Error::FailedToCastBetweenTypesValue);
        Ok(result)
    }
}

/// A representation of a morphism from T to O. This provides some metadata so we can have a single
/// blanket implementation of some T to Some O where Self has some way of converting the morphism.
pub struct ValueMorphism<T, O> {
    pub to_morph: T,
    pub output: PhantomData<O>,
}

impl<T, O> ValueMorphism<T, O> {
    pub fn new(to_morph: T) -> Self {
        ValueMorphism {
            to_morph,
            output: Default::default(),
        }
    }
}

impl<O> TryMorph<ValueMorphism<&mut &[u8], O>> for SubstrateAbiConverter
where
    SubstrateAbiConverter: Convert<u32, O>,
    SubstrateAbiConverter: Convert<u64, O>,
    SubstrateAbiConverter: Convert<u128, O>,
    SubstrateAbiConverter: Convert<U256, O>,
{
    type Outcome = Result<O, Error>;

    fn try_morph(a: ValueMorphism<&mut &[u8], O>) -> Result<Self::Outcome, ()> {
        match a.to_morph.len() {
            4 => {
                let val: Result<Value32, Error> =
                    Decode::decode(a.to_morph).map_err(|_| Error::FailedToCastBetweenTypesValue);

                Ok(val.map(Self::convert))
            }
            8 => {
                let val: Result<Value64, Error> =
                    Decode::decode(a.to_morph).map_err(|_| Error::FailedToCastBetweenTypesValue);
                Ok(val.map(Self::convert))
            }
            16 => {
                let val: Result<Value128, Error> =
                    Decode::decode(a.to_morph).map_err(|_| Error::FailedToCastBetweenTypesValue);
                Ok(val.map(Self::convert))
            }
            32 => {
                let val: Result<Value256, Error> =
                    Decode::decode(a.to_morph).map_err(|_| Error::FailedToCastBetweenTypesValue);
                Ok(val.map(Self::convert))
            }
            _ => Ok(Err(Error::FailedToCastBetweenTypesValue)),
        }
    }
}

impl<O> TryMorph<ValueMorphism<&mut &[u8], Option<O>>> for SubstrateAbiConverter
where
    SubstrateAbiConverter: Convert<u32, O>,
    SubstrateAbiConverter: Convert<u64, O>,
    SubstrateAbiConverter: Convert<u128, O>,
    SubstrateAbiConverter: Convert<U256, O>,
    SubstrateAbiConverter: Convert<U256, O>,
{
    type Outcome = Result<Option<O>, Error>;

    // TODO: make this zero copy
    fn try_morph(a: ValueMorphism<&mut &[u8], Option<O>>) -> Result<Self::Outcome, ()> {
        match a.to_morph.len() {
            1 => Ok(Ok(None)),
            5 => {
                let val: Result<Value32, Error> =
                    Decode::decode(a.to_morph).map_err(|_| Error::FailedToCastBetweenTypesValue);

                Ok(val.map(Self::convert).map(|o| Some(o)))
            }
            9 => {
                let val: Result<Value64, Error> =
                    Decode::decode(a.to_morph).map_err(|_| Error::FailedToCastBetweenTypesValue);
                Ok(val.map(Self::convert).map(|o| Some(o)))
            }
            17 => {
                let val: Result<Value128, Error> =
                    Decode::decode(a.to_morph).map_err(|_| Error::FailedToCastBetweenTypesValue);
                Ok(val.map(Self::convert).map(|o| Some(o)))
            }
            33 => {
                let val: Result<Value256, Error> =
                    Decode::decode(a.to_morph).map_err(|_| Error::FailedToCastBetweenTypesValue);
                Ok(val.map(Self::convert).map(|o| Some(o)))
            }
            _ => Ok(Err(Error::FailedToCastBetweenTypesValue)),
        }
    }
}

impl<T> Convert<T, T> for SubstrateAbiConverter {
    fn convert(a: T) -> T {
        a
    }
}

impl Convert<u32, u64> for SubstrateAbiConverter {
    fn convert(a: u32) -> u64 {
        a as u64
    }
}

impl Convert<u32, u128> for SubstrateAbiConverter {
    fn convert(a: u32) -> u128 {
        a as u128
    }
}

impl Convert<u32, U256> for SubstrateAbiConverter {
    fn convert(a: u32) -> U256 {
        U256::from(a)
    }
}

impl Convert<u64, u32> for SubstrateAbiConverter {
    fn convert(a: u64) -> u32 {
        a as u32
    }
}

impl Convert<u64, u128> for SubstrateAbiConverter {
    fn convert(a: u64) -> u128 {
        a as u128
    }
}

impl Convert<u64, U256> for SubstrateAbiConverter {
    fn convert(a: u64) -> U256 {
        U256::from(a)
    }
}

impl Convert<u128, u32> for SubstrateAbiConverter {
    fn convert(a: u128) -> u32 {
        a as u32
    }
}

impl Convert<u128, u64> for SubstrateAbiConverter {
    fn convert(a: u128) -> u64 {
        a as u64
    }
}

impl Convert<u128, U256> for SubstrateAbiConverter {
    fn convert(a: u128) -> U256 {
        U256::from(a)
    }
}

impl Convert<U256, u32> for SubstrateAbiConverter {
    fn convert(a: U256) -> u32 {
        a.as_u32()
    }
}

impl Convert<U256, u64> for SubstrateAbiConverter {
    fn convert(a: U256) -> u64 {
        a.as_u64()
    }
}

impl Convert<U256, u128> for SubstrateAbiConverter {
    fn convert(a: U256) -> u128 {
        a.as_u128()
    }
}

impl SubstrateAbi for SubstrateAbiConverter {}

// todo: maybe auto-conversion using Boxed types?
// pub trait IValue {
//     fn convert_2(&self, target_len: ValueLen) -> Result<Box<dyn IValue>, SabiError>;
// }
//
// #[derive(Clone, Eq, PartialEq, Debug, TypeInfo)]
// pub struct Value64 {
//     val: u64,
// }
//
// #[derive(Clone, Eq, PartialEq, Debug, TypeInfo)]
// pub struct Value128 {
//     val: u128,
// }
//
// #[derive(Clone, Eq, PartialEq, Debug, TypeInfo)]
// pub struct Value256 {
//     val: U256,
// }
//
// impl Value256 {
//     fn as_u256(&self) -> U256 {
//         self.val
//     }
//
//     fn new(val: U256) -> Value256 {
//         Value256 { val }
//     }
// }
//
// impl Value128 {
//     fn as_u128(&self) -> u128 {
//         self.val
//     }
//
//     fn new(val: u128) -> Value128 {
//         Value128 { val }
//     }
// }
//
// impl Value64 {
//     fn as_u64(&self) -> u64 {
//         self.val
//     }
//
//     fn new(val: u64) -> Value64 {
//         Value64 { val }
//     }
// }

//
// impl IValue for Value64 {
//     fn convert_2(&self, target_len: ValueLen) -> Result<Box<dyn IValue>, SabiError> {
//         match target_len {
//             ValueLen::U64 => Ok(Box::new(self.clone())),
//             ValueLen::U128 => Ok(Box::new(Value128::new(Sabi::value_64_2_value_128(
//                 self.val,
//             )))),
//             ValueLen::U256 => Ok(Box::new(Value256::new(Sabi::value_64_2_value_256(
//                 self.val,
//             )))),
//         }
//     }
// }
//
// impl IValue for Value128 {
//     fn convert_2(&self, target_len: ValueLen) -> Result<Box<dyn IValue>, SabiError> {
//         match target_len {
//             ValueLen::U64 => match Sabi::value_128_2_value_64(self.val) {
//                 Ok(val) => Ok(Box::new(Value64::new(val))),
//                 Err(_) => Err(SabiError::SABIFailedToCastBetweenTypesValue),
//             },
//             ValueLen::U128 => Ok(Box::new(self.clone())),
//             ValueLen::U256 => Ok(Box::new(Value256::new(Sabi::value_128_2_value_256(
//                 self.val,
//             )))),
//         }
//     }
// }
//
// impl IValue for Value256 {
//     fn convert_2(&self, target_len: ValueLen) -> Result<Box<dyn IValue>, SabiError> {
//         match target_len {
//             ValueLen::U64 => match Sabi::value_256_2_value_64(self.val) {
//                 Ok(val) => Ok(Box::new(Value64::new(val))),
//                 Err(_) => Err(SabiError::SABIFailedToCastBetweenTypesValue),
//             },
//             ValueLen::U128 => match Sabi::value_256_2_value_128(self.val) {
//                 Ok(val) => Ok(Box::new(Value128::new(val))),
//                 Err(_) => Err(SabiError::SABIFailedToCastBetweenTypesValue),
//             },
//             ValueLen::U256 => Ok(Box::new(self.clone())),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sabi_decodes_u128_to_target_values_correctly() {
        let input_val: u128 = 88;
        let output_256: U256 = SubstrateAbiConverter::convert(input_val);
        assert_eq!(output_256, U256::from(input_val));
    }

    #[test]
    fn try_morph_bytes_to_u32() {
        let x = 50_u32.encode();
        let morphism = SubstrateAbiConverter::try_morph(ValueMorphism::<_, U256>::new(&mut &x[..]))
            .unwrap()
            .unwrap();
        assert_eq!(morphism, U256::from(50_u32));
    }
}
