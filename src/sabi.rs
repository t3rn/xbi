use codec::{Decode, Encode};

use sp_core::U256;
use sp_std::prelude::*;

use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

/// Global XBI Types.
/// Could also introduce t3rn-primitives/abi but perhaps easier to rely on sp_std / global types
pub type Data = Vec<u8>;
pub type AssetId = u64; // Could also be xcm::MultiAsset
pub type Gas = u64; // [u64; 4]
pub type AccountId32 = sp_runtime::AccountId32;
pub type AccountId20 = sp_core::H160; // Could also take it from MultiLocation::Junction::AccountKey20 { network: NetworkId, key: [u8; 20] },

pub type Value32 = u32;
pub type Value64 = u64;
pub type Value128 = u128;
pub type Value256 = U256;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum SabiError {
    SABIFailedToCastBetweenTypesAddresses,
    SABIFailedToCastBetweenTypesValue,
}

pub enum ValueLen {
    U32,
    U64,
    U128,
    U256,
}

// Substrate ABI - Sabi :)
pub struct Sabi {}

impl Sabi {
    /// Output Accounts
    pub fn account_20_2_account_32(
        account_20: AccountId20,
        extra_bytes: &[u8; 12],
    ) -> Result<AccountId32, SabiError> {
        let mut dest_bytes: Vec<u8> = vec![];
        dest_bytes.append(&mut account_20.encode());
        dest_bytes.append(&mut extra_bytes.encode());

        Decode::decode(&mut &dest_bytes.as_slice()[..])
            .map_err(|_e| SabiError::SABIFailedToCastBetweenTypesValue)
    }

    pub fn account_32_2_account_20(account_32: AccountId32) -> Result<AccountId20, SabiError> {
        let mut dest_bytes: Vec<u8> = vec![];
        let account_32_encoded = account_32.encode();

        for &byte_of_account in account_32_encoded.iter().take(20) {
            dest_bytes.push(byte_of_account);
        }
        Decode::decode(&mut &dest_bytes.as_slice()[..])
            .map_err(|_e| SabiError::SABIFailedToCastBetweenTypesValue)
    }

    /// Output Value256
    pub fn value_32_2_value_256(val: u32) -> U256 {
        U256::from(val)
    }

    pub fn value_64_2_value_256(val: u64) -> U256 {
        U256::from(val)
    }

    pub fn value_128_2_value_256(val: u128) -> U256 {
        U256::from(val)
    }

    pub fn maybe_value_128_2_value_256(maybe_val: Option<u128>) -> U256 {
        match maybe_val {
            None => U256::zero(),
            Some(val) => Self::value_128_2_value_256(val),
        }
    }

    pub fn value_bytes_2_value_256(val_bytes: &Vec<u8>) -> Result<Value256, SabiError> {
        match val_bytes.len() {
            4 => {
                let val_u32: Value32 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(Self::value_32_2_value_256(val_u32))
            },
            8 => {
                let val_u64: Value64 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(Self::value_64_2_value_256(val_u64))
            },
            16 => {
                let val_u128: Value128 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(Self::value_128_2_value_256(val_u128))
            },
            32 => {
                let val_u256: Value256 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(val_u256)
            },
            _ => Err(SabiError::SABIFailedToCastBetweenTypesValue),
        }
    }

    pub fn maybe_value_bytes_2_maybe_value_256(
        val_bytes: &Vec<u8>,
    ) -> Result<Option<Value256>, SabiError> {
        match val_bytes.len() {
            1 => Ok(None),
            5 => {
                let val_u32: Value32 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                let val_u256 = Self::value_32_2_value_256(val_u32);
                Ok(Some(val_u256))
            },
            9 => {
                let val_u64: Value64 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                let val_u256 = Self::value_64_2_value_256(val_u64);
                Ok(Some(val_u256))
            },
            17 => {
                let val_u128: Value128 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                let val_u256 = Self::value_128_2_value_256(val_u128);
                Ok(Some(val_u256))
            },
            33 => {
                let val_u256: Value256 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(Some(val_u256))
            },
            _ => Err(SabiError::SABIFailedToCastBetweenTypesValue),
        }
    }

    /// Output Value128
    pub fn value_32_2_value_128(val: u32) -> u128 {
        val as u128
    }

    pub fn value_64_2_value_128(val: u64) -> u128 {
        val as u128
    }

    pub fn value_256_2_value_128(val: U256) -> Result<u128, SabiError> {
        let val_128 = val.as_u128();
        if val_128 < u128::MAX {
            return Ok(val_128)
        }
        Err(SabiError::SABIFailedToCastBetweenTypesValue)
    }

    pub fn value_bytes_2_value_128(val_bytes: &Vec<u8>) -> Result<u128, SabiError> {
        match val_bytes.len() {
            4 => {
                let val_u32: Value32 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(Self::value_32_2_value_128(val_u32))
            },
            8 => {
                let val_u64: Value64 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(Self::value_64_2_value_128(val_u64))
            },
            16 => {
                let val_u128: Value128 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(val_u128)
            },
            32 => {
                let val_u256: Value256 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Self::value_256_2_value_128(val_u256)
            },
            _ => Err(SabiError::SABIFailedToCastBetweenTypesValue),
        }
    }

    pub fn maybe_value_bytes_2_maybe_value_128(
        val_bytes: &Vec<u8>,
    ) -> Result<Option<Value128>, SabiError> {
        match val_bytes.len() {
            1 => Ok(None),
            5 => {
                let val_u32: Value32 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                let val_u128 = Self::value_32_2_value_128(val_u32);
                Ok(Some(val_u128))
            },
            9 => {
                let val_u64: Value64 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                let val_u128 = Self::value_64_2_value_128(val_u64);

                Ok(Some(val_u128))
            },
            17 => {
                let val_u128: Value128 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                Ok(Some(val_u128))
            },
            33 => {
                let val_u256: Value256 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                let val_u128 = Self::value_256_2_value_128(val_u256)?;
                Ok(Some(val_u128))
            },
            _ => Err(SabiError::SABIFailedToCastBetweenTypesValue),
        }
    }

    /// Output Value64
    pub fn value_32_2_value_64(val: u32) -> u64 {
        val as u64
    }

    pub fn value_128_2_value_64(val: u128) -> Result<u64, SabiError> {
        let val_64 = val as u64;
        if val_64 < u64::MAX {
            return Ok(val_64)
        }
        Err(SabiError::SABIFailedToCastBetweenTypesValue)
    }

    pub fn value_256_2_value_64(val: U256) -> Result<u64, SabiError> {
        let val_64 = val.as_u64();
        if val_64 < u64::MAX {
            return Ok(val_64)
        }
        Err(SabiError::SABIFailedToCastBetweenTypesValue)
    }

    pub fn value_bytes_2_value_64(val_bytes: &Vec<u8>) -> Result<u64, SabiError> {
        match val_bytes.len() {
            4 => {
                let val_u32: Value32 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(Self::value_32_2_value_64(val_u32))
            },
            8 => {
                let val_u64: Value64 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(val_u64)
            },
            16 => {
                let val_u128: Value128 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Self::value_128_2_value_64(val_u128)
            },
            32 => {
                let val_u256: Value256 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Self::value_256_2_value_64(val_u256)
            },
            _ => Err(SabiError::SABIFailedToCastBetweenTypesValue),
        }
    }

    /// Output Value32
    pub fn value_64_2_value_32(val: u64) -> Result<u32, SabiError> {
        let val_32 = val as u32;
        if val_32 < u32::MAX {
            return Ok(val_32)
        }
        Err(SabiError::SABIFailedToCastBetweenTypesValue)
    }

    pub fn value_128_2_value_32(val: u128) -> Result<u32, SabiError> {
        let val_32 = val as u32;
        if val_32 < u32::MAX {
            return Ok(val_32)
        }
        Err(SabiError::SABIFailedToCastBetweenTypesValue)
    }

    pub fn value_256_2_value_32(val: U256) -> Result<u32, SabiError> {
        let val_32 = val.as_u32();
        if val_32 < u32::MAX {
            return Ok(val_32)
        }
        Err(SabiError::SABIFailedToCastBetweenTypesValue)
    }

    pub fn value_bytes_2_value_32(val_bytes: &Vec<u8>) -> Result<u32, SabiError> {
        match val_bytes.len() {
            4 => {
                let val_u32: Value32 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(val_u32)
            },
            8 => {
                let val_u64: Value64 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Self::value_64_2_value_32(val_u64)
            },
            16 => {
                let val_u128: Value128 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Self::value_128_2_value_32(val_u128)
            },
            32 => {
                let val_u256: Value256 = Decode::decode(&mut &val_bytes[..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Self::value_256_2_value_32(val_u256)
            },
            _ => Err(SabiError::SABIFailedToCastBetweenTypesValue),
        }
    }

    pub fn maybe_value_bytes_2_maybe_value_32(
        val_bytes: &Vec<u8>,
    ) -> Result<Option<u32>, SabiError> {
        match val_bytes.len() {
            1 => Ok(None),
            5 => {
                let val_u32: Value32 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                Ok(Some(val_u32))
            },
            9 => {
                let val_u64: Value64 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                let val_u32 = Self::value_64_2_value_32(val_u64)?;
                Ok(Some(val_u32))
            },
            17 => {
                let val_u128: Value128 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;

                let val_u32 = Self::value_128_2_value_32(val_u128)?;
                Ok(Some(val_u32))
            },
            33 => {
                let val_u256: Value256 = Decode::decode(&mut &val_bytes[1..])
                    .map_err(|_| SabiError::SABIFailedToCastBetweenTypesValue)?;
                let val_u32 = Self::value_256_2_value_32(val_u256)?;
                Ok(Some(val_u32))
            },
            _ => Err(SabiError::SABIFailedToCastBetweenTypesValue),
        }
    }
}

#[test]
fn sabi_decodes_u128_to_target_values_correctly() {
    let input_val: u128 = 88;
    let output_256 = Sabi::value_128_2_value_256(input_val);
    assert_eq!(output_256, U256::from(input_val));
}

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
