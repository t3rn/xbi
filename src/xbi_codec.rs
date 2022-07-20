use codec::{Decode, Encode, Input, Output};
use sp_std::prelude::*;

pub use crate::{xbi_abi::*, xbi_format::*};

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
                let mut gas: [u8; 8] = Default::default();
                let mut max_fee_per_gas: [u8; 32] = Default::default();
                input.read(&mut source[..])?;
                input.read(&mut dest[..])?;
                input.read(&mut value[..])?;
                input.read(&mut gas[..])?;
                input.read(&mut max_fee_per_gas[..])?;

                let is_max_priority_fee_per_gas_some = input.read_byte()?;
                let mut max_priority_fee_per_gas = if is_max_priority_fee_per_gas_some == 0u8 {
                    vec![0u8]
                } else {
                    vec![0u8; 33]
                };
                input.read(&mut max_priority_fee_per_gas[..])?;
                // Custom encoder pushed the extra byte for the ease of reading now
                let is_nonce_some = input.read_byte()?;
                let mut nonce = if is_nonce_some == 0 {
                    vec![0u8; 1]
                } else {
                    vec![0u8; 33]
                };
                input.read(&mut nonce[..])?;
                let access_list_len = input.read_byte()?;
                let mut access_list = vec![0u8; access_list_len as usize];
                input.read(&mut access_list[..])?;

                let data_len = input.read_byte()?;
                let mut data = vec![0u8; data_len as usize];
                input.read(&mut data[..])?;

                Ok(XBIInstr::CallEvm {
                    source: AccountId20::from(source),
                    target: AccountId20::from(dest),
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
            4 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // // Minimum length of XBI::CallWasm with empty / none values
                if len? < 92 as usize {
                    return Err("Wrong XBI Order length".into())
                }

                let mut dest: [u8; 32] = Default::default();
                let mut caller: [u8; 32] = Default::default();
                let mut value: [u8; 16] = Default::default();
                input.read(&mut dest[..])?;
                input.read(&mut caller[..])?;
                input.read(&mut value[..])?;

                let data_len = input.read_byte()?;
                let mut data = vec![0u8; data_len as usize];
                input.read(&mut data[..])?;

                let mut limit: [u8; 8] = Default::default();
                input.read(&mut limit[..])?;

                let additional_params_len = input.read_byte()?;
                let mut additional_params = vec![0u8; additional_params_len as usize];
                input.read(&mut additional_params[..])?;

                Ok(XBIInstr::CallCustom {
                    caller: Decode::decode(&mut &caller[..])?,
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                    input: Decode::decode(&mut &data[..])?,
                    limit: Decode::decode(&mut &limit[..])?,
                    additional_params: Decode::decode(&mut &additional_params[..])?,
                })
            },
            5 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                if len? < 48 as usize {
                    return Err("Wrong XBI Order length".into())
                }

                let mut dest: [u8; 32] = Default::default();
                let mut value: [u8; 16] = Default::default();
                input.read(&mut dest[..])?;
                input.read(&mut value[..])?;

                Ok(XBIInstr::Transfer {
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                })
            },
            6 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                if len? < 56 as usize {
                    return Err("Wrong XBI Order length".into())
                }

                let mut currency_id: [u8; 8] = Default::default();
                let mut dest: [u8; 32] = Default::default();
                let mut value: [u8; 16] = Default::default();
                input.read(&mut currency_id[..])?;
                input.read(&mut dest[..])?;
                input.read(&mut value[..])?;

                Ok(XBIInstr::TransferORML {
                    currency_id: Decode::decode(&mut &currency_id[..])?,
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                })
            },
            7 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                if len? < 56 as usize {
                    return Err("Wrong XBI Order length".into())
                }

                let mut currency_id: [u8; 8] = Default::default();
                let mut dest: [u8; 32] = Default::default();
                let mut value: [u8; 16] = Default::default();
                input.read(&mut currency_id[..])?;
                input.read(&mut dest[..])?;
                input.read(&mut value[..])?;

                Ok(XBIInstr::TransferAssets {
                    currency_id: Decode::decode(&mut &currency_id[..])?,
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                })
            },
            8 => {
                let mut outcome: [u8; 1] = Default::default();
                input.read(&mut outcome[..])?;

                let output_len = input.read_byte()?;
                let mut output = vec![0u8; output_len as usize];
                input.read(&mut output[..])?;

                let witness_len = input.read_byte()?;
                let mut witness = vec![0u8; witness_len as usize];
                input.read(&mut witness[..])?;

                Ok(XBIInstr::Result {
                    outcome: Decode::decode(&mut &outcome[..])?,
                    output: Decode::decode(&mut &output[..])?,
                    witness: Decode::decode(&mut &witness[..])?,
                })
            },
            9 => {
                let mut kind: [u8; 1] = Default::default();
                input.read(&mut kind[..])?;

                let instruction_id_len = input.read_byte()?;
                let mut instruction_id = vec![0u8; instruction_id_len as usize];
                input.read(&mut instruction_id[..])?;

                let extra_len = input.read_byte()?;
                let mut extra = vec![0u8; extra_len as usize];
                input.read(&mut extra[..])?;

                Ok(XBIInstr::Notification {
                    kind: Decode::decode(&mut &kind[..])?,
                    instruction_id: Decode::decode(&mut &instruction_id[..])?,
                    extra: Decode::decode(&mut &extra[..])?,
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
                target,
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
                target.encode_to(dest_bytes);
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
                limit,
                additional_params,
            } => {
                dest_bytes.push_byte(4);
                caller.encode_to(dest_bytes);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
                dest_bytes.push_byte(input.encode().len() as u8);
                input.encode_to(dest_bytes);
                limit.encode_to(dest_bytes);
                dest_bytes.push_byte(additional_params.encode().len() as u8);
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
                dest_bytes.push_byte(output.encode().len() as u8);
                output.encode_to(dest_bytes);
                dest_bytes.push_byte(witness.encode().len() as u8);
                witness.encode_to(dest_bytes);
            },
            XBIInstr::Notification {
                kind,
                instruction_id,
                extra,
            } => {
                dest_bytes.push_byte(9);
                kind.encode_to(dest_bytes);
                dest_bytes.push_byte(instruction_id.encode().len() as u8);
                instruction_id.encode_to(dest_bytes);
                dest_bytes.push_byte(extra.encode().len() as u8);
                extra.encode_to(dest_bytes);
            },
        }
    }
}
