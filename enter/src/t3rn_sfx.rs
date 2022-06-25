use crate::Error;
use codec::Decode;
use pallet_xbi_portal::xbi_codec::{XBIInstr, XBIMetadata};

use pallet_xbi_portal::xbi_format::XBIFormat;
use t3rn_primitives::{side_effect::SideEffect, transfers::EscrowedBalanceOf, EscrowTrait};

pub fn from_t3rn_sfx_2_xbi<T: frame_system::Config, E: EscrowTrait<T>>(
    side_effect: SideEffect<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::BlockNumber,
        EscrowedBalanceOf<T, E>,
    >,
    metadata: XBIMetadata,
) -> Result<XBIFormat, Error<T>> {
    match &side_effect.encoded_action[0..4] {
        b"tran" => Ok(XBIFormat {
            instr: XBIInstr::Transfer {
                // Get dest as argument_1 of SFX::Transfer of Type::DynamicAddress
                dest: Decode::decode(&mut &side_effect.encoded_args[1][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_2 of SFX::Transfer of Type::Value
                value: Decode::decode(&mut &side_effect.encoded_args[2][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
            },
            metadata,
        }),
        b"mult" | b"tass" => Ok(XBIFormat {
            instr: XBIInstr::TransferAssets {
                // Get dest as argument_0 of SFX::TransferAssets of Type::DynamicBytes
                currency_id: Decode::decode(&mut &side_effect.encoded_args[0][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_1 of SFX::TransferAssets of Type::DynamicAddress
                dest: Decode::decode(&mut &side_effect.encoded_args[1][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_2 of SFX::TransferAssets of Type::Value
                value: Decode::decode(&mut &side_effect.encoded_args[2][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
            },
            metadata,
        }),
        b"orml" => Ok(XBIFormat {
            instr: XBIInstr::TransferORML {
                // Get dest as argument_0 of SFX::TransferOrml of Type::DynamicBytes
                currency_id: Decode::decode(&mut &side_effect.encoded_args[0][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_1 of SFX::TransferOrml of Type::DynamicAddress
                dest: Decode::decode(&mut &side_effect.encoded_args[1][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_2 of SFX::TransferOrml of Type::Value
                value: Decode::decode(&mut &side_effect.encoded_args[2][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
            },
            metadata,
        }),
        b"swap" => Err(Error::<T>::EnterSfxNotRecognized),
        b"aliq" => Err(Error::<T>::EnterSfxNotRecognized),
        b"cevm" => Ok(XBIFormat {
            instr: XBIInstr::CallEvm {
                // Get dest as argument_0 of SFX::CallEvm of Type::DynamicAddress
                source: Decode::decode(&mut &side_effect.encoded_args[0][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_1 of SFX::CallEvm of Type::DynamicAddress
                dest: Decode::decode(&mut &side_effect.encoded_args[1][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_2 of SFX::CallEvm of Type::Value
                value: Decode::decode(&mut &side_effect.encoded_args[2][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_3 of SFX::CallEvm of Type::DynamicBytes
                input: Decode::decode(&mut &side_effect.encoded_args[3][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingDataErr)?,
                // Get dest as argument_4 of SFX::CallEvm of Type::Uint(64)
                gas_limit: Decode::decode(&mut &side_effect.encoded_args[4][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_5 of SFX::CallEvm of Type::Value
                max_fee_per_gas: Decode::decode(&mut &side_effect.encoded_args[5][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_6 of SFX::CallEvm of Type::Option(Box::from(Type::Value))
                max_priority_fee_per_gas: Decode::decode(&mut &side_effect.encoded_args[6][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_7 of SFX::CallEvm of Type::Option(Box::from(Type::Value))
                nonce: Decode::decode(&mut &side_effect.encoded_args[7][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_8 of SFX::CallEvm of Type::DynamicBytes
                access_list: Decode::decode(&mut &side_effect.encoded_args[8][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingDataErr)?,
            },
            metadata,
        }),
        b"wasm" => Ok(XBIFormat {
            instr: XBIInstr::CallWasm {
                // Get dest as argument_0 of SFX::CallWasm of Type::DynamicAddress
                dest: Decode::decode(&mut &side_effect.encoded_args[0][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_1 of SFX::CallWasm of Type::Value
                value: Decode::decode(&mut &side_effect.encoded_args[1][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_2 of SFX::CallWasm of Type::Value
                gas_limit: Decode::decode(&mut &side_effect.encoded_args[2][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_3 of SFX::CallEvm of Type::Option(Box::from(Type::Value))
                storage_deposit_limit: Decode::decode(&mut &side_effect.encoded_args[3][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_4 of SFX::CallEvm of Type::DynamicBytes
                data: Decode::decode(&mut &side_effect.encoded_args[4][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingDataErr)?,
            },
            metadata,
        }),
        b"comp" => Err(Error::<T>::EnterSfxNotRecognized),
        b"call" => Ok(XBIFormat {
            instr: XBIInstr::CallCustom {
                // Get dest as argument_0 of SFX::CallWasm of Type::DynamicAddress
                caller: Decode::decode(&mut &side_effect.encoded_args[0][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_1 of SFX::CallWasm of Type::DynamicAddress
                dest: Decode::decode(&mut &side_effect.encoded_args[1][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingAddressErr)?,
                // Get dest as argument_2 of SFX::CallWasm of Type::Value
                value: Decode::decode(&mut &side_effect.encoded_args[2][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_3 of SFX::CallEvm of Type::DynamicBytes
                input: Decode::decode(&mut &side_effect.encoded_args[3][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingDataErr)?,
                // Get dest as argument_4 of SFX::CallWasm of Type::Value
                limit: Decode::decode(&mut &side_effect.encoded_args[4][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
                // Get dest as argument_5 of SFX::CallEvm of Type::DynamicBytes
                additional_params: Decode::decode(&mut &side_effect.encoded_args[5][..])
                    .map_err(|_| Error::<T>::EnterSfxDecodingValueErr)?,
            },
            metadata,
        }),
        &_ => Err(Error::<T>::EnterSfxNotRecognized),
    }
}
