use codec::{Decode, Encode, Input, Output};
use std::process::id;

pub use crate::*;

impl Decode for XbiInstruction {
    fn decode<I: Input>(input: &mut I) -> Result<Self, codec::Error> {
        let b = input.read_byte()?;

        match b {
            0 => Err("Unknown XBI Order".into()),
            1 => match input.remaining_len()? {
                Some(remaining_len) => {
                    let mut payload = vec![0u8; remaining_len];
                    input.read(&mut payload[..])?;
                    Ok(XbiInstruction::CallNative { payload })
                }
                None => Err("Wrong XBI Order length".into()),
            },
            2 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallEvm with empty / none values
                if len? < 104_usize {
                    return Err("Wrong XBI Order length".into());
                }

                let mut source: [u8; 20] = Default::default();
                let mut dest: [u8; 20] = Default::default();
                let mut value: [u8; 32] = Default::default();
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

                Ok(XbiInstruction::CallEvm {
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
            }
            3 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                if len? < 60_usize {
                    return Err("Wrong XBI Order length".into());
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

                Ok(XbiInstruction::CallWasm {
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                    gas_limit: Decode::decode(&mut &gas[..])?,
                    storage_deposit_limit: Decode::decode(&mut &storage_deposit_limit[..])?,
                    data: Decode::decode(&mut &data[..])?,
                })
            }
            4 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // // Minimum length of XBI::CallWasm with empty / none values
                if len? < 92_usize {
                    return Err("Wrong XBI Order length".into());
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

                Ok(XbiInstruction::CallCustom {
                    caller: Decode::decode(&mut &caller[..])?,
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                    input: Decode::decode(&mut &data[..])?,
                    limit: Decode::decode(&mut &limit[..])?,
                    additional_params: Decode::decode(&mut &additional_params[..])?,
                })
            }
            5 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                if len? < 48_usize {
                    return Err("Wrong XBI Order length".into());
                }

                let mut dest: [u8; 32] = Default::default();
                let mut value: [u8; 16] = Default::default();
                input.read(&mut dest[..])?;
                input.read(&mut value[..])?;

                Ok(XbiInstruction::Transfer {
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                })
            }
            6 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                if len? < 52_usize {
                    return Err("Wrong XBI Order length".into());
                }

                let mut currency_id: [u8; 4] = Default::default();
                let mut dest: [u8; 32] = Default::default();
                let mut value: [u8; 16] = Default::default();
                input.read(&mut currency_id[..])?;
                input.read(&mut dest[..])?;
                input.read(&mut value[..])?;

                Ok(XbiInstruction::TransferAssets {
                    currency_id: Decode::decode(&mut &currency_id[..])?,
                    dest: Decode::decode(&mut &dest[..])?,
                    value: Decode::decode(&mut &value[..])?,
                })
            }
            // Swap
            7 => {
                let _len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                // if len? < 52_usize {
                //     return Err("Wrong XBI Order length".into());
                // }

                let mut asset_out: [u8; 4] = Default::default();
                let mut asset_in: [u8; 4] = Default::default();
                let mut amount: [u8; 16] = Default::default();
                let mut max_limit: [u8; 16] = Default::default();
                let mut discount: [u8; 1] = Default::default();
                input.read(&mut asset_out[..])?;
                input.read(&mut asset_in[..])?;
                input.read(&mut amount[..])?;
                input.read(&mut max_limit[..])?;
                input.read(&mut discount[..])?;

                Ok(XbiInstruction::Swap {
                    asset_out: Decode::decode(&mut &asset_out[..])?,
                    asset_in: Decode::decode(&mut &asset_in[..])?,
                    amount: Decode::decode(&mut &amount[..])?,
                    max_limit: Decode::decode(&mut &max_limit[..])?,
                    discount: Decode::decode(&mut &discount[..])?,
                })
            }
            // AddLiquidity
            8 => {
                let _len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                // if len? < 52_usize {
                //     return Err("Wrong XBI Order length".into());
                // }

                let mut asset_a: [u8; 4] = Default::default();
                let mut asset_b: [u8; 4] = Default::default();
                let mut amount_a: [u8; 16] = Default::default();
                let mut amount_b_max_limit: [u8; 16] = Default::default();
                input.read(&mut asset_a[..])?;
                input.read(&mut asset_b[..])?;
                input.read(&mut amount_a[..])?;
                input.read(&mut amount_b_max_limit[..])?;

                Ok(XbiInstruction::AddLiquidity {
                    asset_a: Decode::decode(&mut &asset_a[..])?,
                    asset_b: Decode::decode(&mut &asset_b[..])?,
                    amount_a: Decode::decode(&mut &amount_a[..])?,
                    amount_b_max_limit: Decode::decode(&mut &amount_b_max_limit[..])?,
                })
            }
            // RemoveLiquidity
            9 => {
                let _len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                // if len? < 52_usize {
                //     return Err("Wrong XBI Order length".into());
                // }

                let mut asset_a: [u8; 4] = Default::default();
                let mut asset_b: [u8; 4] = Default::default();
                let mut liquidity_amount: [u8; 16] = Default::default();
                input.read(&mut asset_a[..])?;
                input.read(&mut asset_b[..])?;
                input.read(&mut liquidity_amount[..])?;

                Ok(XbiInstruction::RemoveLiquidity {
                    asset_a: Decode::decode(&mut &asset_a[..])?,
                    asset_b: Decode::decode(&mut &asset_b[..])?,
                    liquidity_amount: Decode::decode(&mut &liquidity_amount[..])?,
                })
            }
            // GetPrice
            10 => {
                let len: Result<usize, codec::Error> = match input.remaining_len()? {
                    Some(remaining_len) => Ok(remaining_len),
                    None => Err("Wrong XBI Order length".into()),
                };

                // Minimum length of XBI::CallWasm with empty / none values
                // TODO: make all these unmagic and provide trait for them
                if len? < 52_usize {
                    return Err("Wrong XBI Order length".into());
                }

                let mut asset_a: [u8; 4] = Default::default();
                let mut asset_b: [u8; 4] = Default::default();
                let mut amount: [u8; 16] = Default::default();
                input.read(&mut asset_a[..])?;
                input.read(&mut asset_b[..])?;
                input.read(&mut amount[..])?;

                Ok(XbiInstruction::GetPrice {
                    asset_a: Decode::decode(&mut &asset_a[..])?,
                    asset_b: Decode::decode(&mut &asset_b[..])?,
                    amount: Decode::decode(&mut &amount[..])?,
                })
            }
            255 => {
                let id_len = input.read_byte()?;
                let mut id = vec![0u8; id_len as usize];
                input.read(&mut id[..])?;

                let mut outcome: [u8; 1] = Default::default();
                input.read(&mut outcome[..])?;

                let output_len = input.read_byte()?;
                let mut output = vec![0u8; output_len as usize];
                input.read(&mut output[..])?;

                let witness_len = input.read_byte()?;
                let mut witness = vec![0u8; witness_len as usize];
                input.read(&mut witness[..])?;

                Ok(XbiInstruction::Result(XbiResult {
                    id: Decode::decode(&mut &id[..])?,
                    status: Decode::decode(&mut &outcome[..])?,
                    output: Decode::decode(&mut &output[..])?,
                    witness: Decode::decode(&mut &witness[..])?,
                }))
            }
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
impl Encode for XbiInstruction {
    fn encode_to<T: Output + ?Sized>(&self, dest_bytes: &mut T) {
        match self {
            XbiInstruction::Unknown { identifier, params } => {
                dest_bytes.push_byte(*identifier);
                params.encode_to(dest_bytes);
            }
            XbiInstruction::CallNative { payload } => {
                dest_bytes.push_byte(1);
                payload.encode_to(dest_bytes);
            }
            XbiInstruction::CallEvm {
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
            }
            XbiInstruction::CallWasm {
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
            }
            XbiInstruction::CallCustom {
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
            }
            XbiInstruction::Transfer { dest, value } => {
                dest_bytes.push_byte(5);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
            }
            XbiInstruction::TransferAssets {
                currency_id,
                dest,
                value,
            } => {
                dest_bytes.push_byte(6);
                currency_id.encode_to(dest_bytes);
                dest.encode_to(dest_bytes);
                value.encode_to(dest_bytes);
            }
            XbiInstruction::Swap {
                asset_out,
                asset_in,
                amount,
                max_limit,
                discount,
            } => {
                dest_bytes.push_byte(7);
                asset_out.encode_to(dest_bytes);
                asset_in.encode_to(dest_bytes);
                amount.encode_to(dest_bytes);
                max_limit.encode_to(dest_bytes);
                discount.encode_to(dest_bytes);
            }
            XbiInstruction::AddLiquidity {
                asset_a,
                asset_b,
                amount_a,
                amount_b_max_limit,
            } => {
                dest_bytes.push_byte(8);
                asset_a.encode_to(dest_bytes);
                asset_b.encode_to(dest_bytes);
                amount_a.encode_to(dest_bytes);
                amount_b_max_limit.encode_to(dest_bytes);
            }
            XbiInstruction::RemoveLiquidity {
                asset_a,
                asset_b,
                liquidity_amount,
            } => {
                dest_bytes.push_byte(9);
                asset_a.encode_to(dest_bytes);
                asset_b.encode_to(dest_bytes);
                liquidity_amount.encode_to(dest_bytes);
            }
            XbiInstruction::GetPrice {
                asset_a,
                asset_b,
                amount,
            } => {
                dest_bytes.push_byte(10);
                asset_a.encode_to(dest_bytes);
                asset_b.encode_to(dest_bytes);
                amount.encode_to(dest_bytes);
            }
            XbiInstruction::Result(XbiResult {
                id,
                status,
                output,
                witness,
            }) => {
                dest_bytes.push_byte(255);

                dest_bytes.push_byte(id.encode().len() as u8);
                id.encode_to(dest_bytes);

                status.encode_to(dest_bytes);

                dest_bytes.push_byte(output.encode().len() as u8);
                output.encode_to(dest_bytes);

                dest_bytes.push_byte(witness.encode().len() as u8);
                witness.encode_to(dest_bytes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{ActionNotificationTimeouts, XbiFormat, XbiMetadata};
    use crate::{XbiCheckOutStatus, XbiInstruction};
    use codec::{Decode, Encode};
    use frame_support::{Blake2_256, StorageHasher};

    #[test]
    fn custom_encodes_decodes_xbi_evm() {
        let xbi_evm = XbiInstruction::CallEvm {
            source: AccountId20::repeat_byte(3),
            target: AccountId20::repeat_byte(2),
            value: sp_core::U256([1, 0, 0, 0]),
            input: vec![8, 9],
            gas_limit: 2,
            max_fee_per_gas: sp_core::U256([4, 5, 6, 7]),
            max_priority_fee_per_gas: None,
            nonce: Some(sp_core::U256([3, 4, 6, 7])),
            access_list: vec![],
        };

        let decoded_xbi_evm: XbiInstruction = Decode::decode(&mut &xbi_evm.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_evm.encode(), xbi_evm.encode());
        assert_eq!(xbi_evm, decoded_xbi_evm);
    }

    #[test]
    fn custom_encodes_decodes_xbi_evm_and_metadata() {
        let xbi_evm_format = XbiFormat {
            instr: XbiInstruction::CallEvm {
                source: AccountId20::repeat_byte(3),
                target: AccountId20::repeat_byte(2),
                value: sp_core::U256([1, 0, 0, 0]),
                input: vec![8, 9],
                gas_limit: 2,
                max_fee_per_gas: sp_core::U256([4, 5, 6, 7]),
                max_priority_fee_per_gas: None,
                nonce: Some(sp_core::U256([3, 4, 6, 7])),
                access_list: vec![],
            },
            metadata: XbiMetadata {
                id: sp_core::H256::repeat_byte(2),
                dest_para_id: 3u32,
                src_para_id: 4u32,
                sent: ActionNotificationTimeouts {
                    action: 1u32,
                    notification: 2u32,
                },
                delivered: ActionNotificationTimeouts {
                    action: 3u32,
                    notification: 4u32,
                },
                executed: ActionNotificationTimeouts {
                    action: 4u32,
                    notification: 5u32,
                },
                max_exec_cost: 6u128,
                max_notifications_cost: 8u128,
                maybe_known_origin: None,
                actual_aggregated_cost: None,
                maybe_fee_asset_id: None,
            },
        };

        let decoded_xbi_evm: XbiFormat = Decode::decode(&mut &xbi_evm_format.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_evm.encode(), xbi_evm_format.encode());
        assert_eq!(xbi_evm_format, decoded_xbi_evm);
    }

    #[test]
    fn custom_encodes_decodes_empty_xbi_evm() {
        let xbi_evm = XbiInstruction::CallEvm {
            source: AccountId20::repeat_byte(3),
            target: AccountId20::repeat_byte(2),
            value: sp_core::U256([1, 0, 0, 0]),
            input: vec![],
            gas_limit: 2,
            max_fee_per_gas: sp_core::U256([4, 5, 6, 7]),
            max_priority_fee_per_gas: None,
            nonce: None,
            access_list: vec![],
        };

        let decoded_xbi_evm: XbiInstruction = Decode::decode(&mut &xbi_evm.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_evm.encode(), xbi_evm.encode());
        assert_eq!(xbi_evm, decoded_xbi_evm);
        assert_eq!(xbi_evm.encode().len(), 121);
    }

    #[test]
    fn custom_encodes_decodes_xbi_wasm() {
        let xbi_wasm = XbiInstruction::CallWasm {
            dest: AccountId32::new([
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ]),
            value: 1,
            gas_limit: 2,
            storage_deposit_limit: Some(6),
            data: vec![8, 9],
        };

        let decoded_xbi_wasm: XbiInstruction = Decode::decode(&mut &xbi_wasm.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_wasm.encode(), xbi_wasm.encode());
        assert_eq!(xbi_wasm, decoded_xbi_wasm);
    }

    #[test]
    fn custom_encodes_decodes_empty_xbi_wasm() {
        let xbi_wasm = XbiInstruction::CallWasm {
            dest: AccountId32::new([
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ]),
            value: 1,
            gas_limit: 2,
            storage_deposit_limit: None,
            data: vec![],
        };

        let decoded_xbi_wasm: XbiInstruction = Decode::decode(&mut &xbi_wasm.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_wasm.encode(), xbi_wasm.encode());
        assert_eq!(xbi_wasm, decoded_xbi_wasm);

        // Minimum length of XBI::CallWasm with empty / none values
        assert_eq!(xbi_wasm.encode().len(), 61);
    }

    #[test]
    fn custom_encodes_decodes_xbi_call_custom() {
        let xbi_call_custom = XbiInstruction::CallCustom {
            caller: AccountId32::new([
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ]),
            dest: AccountId32::new([
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ]),
            value: 1,
            input: vec![8, 9],
            limit: 1,
            additional_params: vec![10u8, 11u8],
        };

        let decoded_xbi_call_custom: XbiInstruction =
            Decode::decode(&mut &xbi_call_custom.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_call_custom.encode(), xbi_call_custom.encode());
        assert_eq!(xbi_call_custom, decoded_xbi_call_custom);
    }

    #[test]
    fn custom_encodes_decodes_empty_xbi_call_custom() {
        let xbi_call_custom = XbiInstruction::CallCustom {
            caller: AccountId32::new([
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ]),
            dest: AccountId32::new([
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ]),
            value: 1,
            input: vec![],
            limit: 1,
            additional_params: vec![],
        };

        let decoded_xbi_call_custom: XbiInstruction =
            Decode::decode(&mut &xbi_call_custom.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_call_custom.encode(), xbi_call_custom.encode());
        assert_eq!(xbi_call_custom, decoded_xbi_call_custom);
        assert_eq!(xbi_call_custom.encode().len(), 93);
    }

    #[test]
    fn custom_encodes_decodes_xbi_transfer() {
        let xbi_transfer = XbiInstruction::Transfer {
            dest: AccountId32::new([
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ]),
            value: 1,
        };

        let decoded_xbi_transfer: XbiInstruction =
            Decode::decode(&mut &xbi_transfer.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_transfer.encode(), xbi_transfer.encode());
        assert_eq!(xbi_transfer, decoded_xbi_transfer);
        assert_eq!(xbi_transfer.encode().len(), 49);
    }

    #[test]
    fn custom_encodes_decodes_xbi_transfer_assets() {
        let xbi_transfer_assets = XbiInstruction::TransferAssets {
            currency_id: 1u32,
            dest: AccountId32::new([
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ]),
            value: 1,
        };

        let decoded_xbi_transfer_assets: XbiInstruction =
            Decode::decode(&mut &xbi_transfer_assets.encode()[..]).unwrap();
        assert_eq!(
            decoded_xbi_transfer_assets.encode(),
            xbi_transfer_assets.encode()
        );
        assert_eq!(xbi_transfer_assets, decoded_xbi_transfer_assets);
        assert_eq!(xbi_transfer_assets.encode().len(), 53);
    }

    #[test]
    fn custom_encodes_decodes_xbi_results() {
        let xbi_result = XbiInstruction::Result(XbiResult {
            id: Blake2_256::hash("helloworldthisismyid".as_bytes()).encode(),
            status: XbiCheckOutStatus::SuccessfullyExecuted,
            output: vec![1, 2, 3],
            witness: vec![4, 5, 6],
        });

        let decoded_xbi_result: XbiInstruction =
            Decode::decode(&mut &xbi_result.encode()[..]).unwrap();
        assert_eq!(decoded_xbi_result.encode(), xbi_result.encode());
        assert_eq!(xbi_result, decoded_xbi_result);
    }
}
