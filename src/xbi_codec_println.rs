use codec::{Decode, Encode, Input, Output};
use sp_std::prelude::*;

pub use crate::xbi_abi::*;
pub use crate::xbi_format::*;

impl Decode for XBIInstr {
    fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
        let b = input.read_byte()?;

        match b {
            0 => Err("Unknown XBI Order".into()),
            1 => match input.remaining_len()? {
                Some(remaining_len) => {
                    let mut payload = vec![0u8; remaining_len];
                    input.read(&mut payload[..])?;
                    Ok(XBIInstr::CallNative { payload })
                },
                None => Err("Wrong XBI Order length".into()),
            },
            2 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallEvm with empty / none values
                if len? < 104 as usize {
                    return Err("Wrong XBI Order length".into())
                }

                let mut source: [u8; 20] = Default::default();
                let mut dest: [u8; 20] = Default::default();
                let mut value: [u8; 16] = Default::default();
                input.read(&mut source[..])?;
                input.read(&mut dest[..])?;
                input.read(&mut value[..])?;
                println!("source {:?}", source);
                println!("dest {:?}", dest);
                println!("value {:?}", value);

                // let data_len = input.read_byte()?;
                // Burn one more byte if vector is non-empty
                // let _ = input.read_byte()?;
                let mut gas: [u8; 8] = Default::default();
                input.read(&mut gas[..])?;
                println!("gas {:?}", gas);

                let mut max_fee_per_gas: [u8; 32] = Default::default();
                input.read(&mut max_fee_per_gas[..])?;
                println!("max_fee_per_gas {:?}", max_fee_per_gas);

                let is_max_priority_fee_per_gas_some = input.read_byte()?;
                println!("is_max_priority_fee_per_gas_some {:?}", is_max_priority_fee_per_gas_some);
                let mut max_priority_fee_per_gas = if is_max_priority_fee_per_gas_some == 0u8 {
                    vec![0u8]
                } else {
                    vec![0u8; 33]
                };
                input.read(&mut max_priority_fee_per_gas[..])?;
                println!("max_priority_fee_per_gas {:?}", max_priority_fee_per_gas);
                // Custom encoder pushed the extra byte for the ease of reading now
                let is_nonce_some = input.read_byte()?;
                println!("is_nonce_some {:?}", is_nonce_some);
                let mut nonce = if is_nonce_some == 0 {
                    vec![0u8; 1]
                } else {
                    vec![0u8; 33]
                    // let mut nonce_tmp = vec![0u8; 32];
                    // let mut is_some_byte = vec![1u8];
                    // input.read(&mut nonce_tmp[..])?;
                    // is_some_byte.append(&mut nonce_tmp);
                    // is_some_byte
                };
                input.read(&mut nonce[..])?;
                println!("nonce {:?}", nonce);

                println!("remaining remaining_len {:?}", input.remaining_len()?);

                let access_list_len = input.read_byte()?;
                println!("access_list_len {:?}", access_list_len);

                println!("remaining remaining_len {:?}", input.remaining_len()?);

                let mut access_list = vec![0u8; access_list_len as usize];
                input.read(&mut access_list[..])?;

                println!("access_list {:?}", access_list);

                let data_len = input.read_byte()?;
                println!("data_len {:?}", data_len);
                println!("remaining remaining_len {:?}", input.remaining_len()?);

                let mut data = vec![0u8; data_len as usize];
                input.read(&mut data[..])?;

                println!("data {:?}", data);
                println!("remaining remaining_len {:?}", input.remaining_len()?);

                Ok(XBIInstr::CallEvm {
                    source: AccountId20::from(source),
                    dest: AccountId20::from(dest),
                    value: Decode::decode(&mut &value[..])?,
                    input: Decode::decode(&mut &data[..])?,
                    gas_limit: Decode::decode(&mut &gas[..])?,
                    max_fee_per_gas: Decode::decode(&mut &max_fee_per_gas[..])?,
                    max_priority_fee_per_gas: Decode::decode(&mut &max_priority_fee_per_gas[..])?,
                    nonce: Decode::decode(&mut &nonce[..])?,
                    access_list: Decode::decode(&mut &access_list[..])?,
                })
            },
            3 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                if len? < 60 as usize {
                    return Err("Wrong XBI Order length".into())
                }

                let mut dest: [u8; 32] = Default::default();
                let mut value: [u8; 16] = Default::default();
                let mut gas: [u8; 8] = Default::default();
                input.read(&mut dest[..])?;
                input.read(&mut value[..])?;
                input.read(&mut gas[..])?;

                let is_storage_deposit_limit_some = input.read_byte()?;
                let mut storage_deposit_limit = if is_storage_deposit_limit_some == 0u8 {
                    vec![0u8]
                } else {
                    vec![0u8; 17]
                };
                input.read(&mut storage_deposit_limit[..])?;

                let data_len = input.read_byte()?;
                let mut data = vec![0u8; data_len as usize];
                input.read(&mut data[..])?;

                Ok(XBIInstr::CallWasm {
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                    gas_limit: Decode::decode(&mut &gas[..])?,
                    storage_deposit_limit: Decode::decode(&mut &storage_deposit_limit[..])?,
                    data: Decode::decode(&mut &data[..])?,
                })
            },
            _ => Err("Unknown XBI Order".into()),
        }
    }
}

/*
 * Encoding of XBI instruction is defined as follows:
 *  identifier / u8 - 255 - extended.
 *  (extended-identifier) - u16 // optional, only read if `identifier==255`
 *  scale-encoded params // bytes
 */
impl Encode for XBIInstr {
    fn encode_to<T: Output + ?Sized>(&self, dest_bytes: &mut T) {
        match &*self {
            XBIInstr::Unknown { identifier, params } => {
                dest_bytes.push_byte(identifier.clone());
                params.encode_to(dest_bytes);
            },
            XBIInstr::CallNative { payload } => {
                dest_bytes.push_byte(1);
                payload.encode_to(dest_bytes);
            },
            XBIInstr::CallEvm {
                source,
                dest,
                value,
                input,
                gas_limit,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list,
            } => {
                dest_bytes.push_byte(2);
                source.encode_to(dest_bytes);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
                gas_limit.encode_to(dest_bytes);
                max_fee_per_gas.encode_to(dest_bytes);
                dest_bytes.push_byte(max_priority_fee_per_gas.is_some() as u8);
                max_priority_fee_per_gas.encode_to(dest_bytes);
                dest_bytes.push_byte(nonce.is_some() as u8);
                nonce.encode_to(dest_bytes);
                dest_bytes.push_byte(access_list.encode().len() as u8);
                access_list.encode_to(dest_bytes);
                dest_bytes.push_byte(input.encode().len() as u8);
                input.encode_to(dest_bytes);
            },
            XBIInstr::CallWasm {
                dest,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
            } => {
                dest_bytes.push_byte(3);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
                gas_limit.encode_to(dest_bytes);
                dest_bytes.push_byte(storage_deposit_limit.is_some() as u8);
                storage_deposit_limit.encode_to(dest_bytes);
                dest_bytes.push_byte(data.encode().len() as u8);
                data.encode_to(dest_bytes);
            },
            XBIInstr::CallCustom {
                caller,
                dest,
                value,
                input,
                additional_params,
            } => {
                dest_bytes.push_byte(4);
                caller.encode_to(dest_bytes);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
                input.encode_to(dest_bytes);
                additional_params.encode_to(dest_bytes);
            },
            XBIInstr::Transfer { dest, value } => {
                dest_bytes.push_byte(5);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
            },
            XBIInstr::TransferORML {
                currency_id,
                dest,
                value,
            } => {
                dest_bytes.push_byte(6);
                currency_id.encode_to(dest_bytes);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
            },
            XBIInstr::TransferAssets {
                currency_id,
                dest,
                value,
            } => {
                dest_bytes.push_byte(7);
                currency_id.encode_to(dest_bytes);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
            },
            XBIInstr::Result {
                outcome,
                output,
                witness,
            } => {
                dest_bytes.push_byte(8);
                outcome.encode_to(dest_bytes);
                output.encode_to(dest_bytes);
                witness.encode_to(dest_bytes);
            },
            XBIInstr::Notification {
                kind,
                instruction_id,
                extra,
            } => {
                dest_bytes.push_byte(9);
                kind.encode_to(dest_bytes);
                instruction_id.encode_to(dest_bytes);
                extra.encode_to(dest_bytes);
            },
        }
    }
}
