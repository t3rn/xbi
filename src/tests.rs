use crate::{xbi_abi::*, xbi_format::XBIInstr};

use crate::{
    xbi_codec::{ActionNotificationTimeouts, XBIFormat, XBIMetadata},
    xbi_format::{XBICheckOutStatus, XBINotificationKind},
};
use codec::{Decode, Encode};

#[test]
fn custom_encodes_decodes_xbi_evm() {
    let xbi_evm = XBIInstr::CallEvm {
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

    let decoded_xbi_evm: XBIInstr = Decode::decode(&mut &xbi_evm.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_evm.encode(), xbi_evm.encode());
    assert_eq!(xbi_evm, decoded_xbi_evm);
}

#[test]
fn custom_encodes_decodes_xbi_evm_and_metadata() {
    let xbi_evm_format = XBIFormat {
        instr: XBIInstr::CallEvm {
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
        metadata: XBIMetadata {
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

    let decoded_xbi_evm: XBIFormat = Decode::decode(&mut &xbi_evm_format.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_evm.encode(), xbi_evm_format.encode());
    assert_eq!(xbi_evm_format, decoded_xbi_evm);
}

#[test]
fn custom_encodes_decodes_empty_xbi_evm() {
    let xbi_evm = XBIInstr::CallEvm {
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

    let decoded_xbi_evm: XBIInstr = Decode::decode(&mut &xbi_evm.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_evm.encode(), xbi_evm.encode());
    assert_eq!(xbi_evm, decoded_xbi_evm);
    assert_eq!(xbi_evm.encode().len(), 121);
}

#[test]
fn custom_encodes_decodes_xbi_wasm() {
    let xbi_wasm = XBIInstr::CallWasm {
        dest: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        value: 1,
        gas_limit: 2,
        storage_deposit_limit: Some(6),
        data: vec![8, 9],
    };

    let decoded_xbi_wasm: XBIInstr = Decode::decode(&mut &xbi_wasm.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_wasm.encode(), xbi_wasm.encode());
    assert_eq!(xbi_wasm, decoded_xbi_wasm);
}

#[test]
fn custom_encodes_decodes_empty_xbi_wasm() {
    let xbi_wasm = XBIInstr::CallWasm {
        dest: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        value: 1,
        gas_limit: 2,
        storage_deposit_limit: None,
        data: vec![],
    };

    let decoded_xbi_wasm: XBIInstr = Decode::decode(&mut &xbi_wasm.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_wasm.encode(), xbi_wasm.encode());
    assert_eq!(xbi_wasm, decoded_xbi_wasm);

    // Minimum length of XBI::CallWasm with empty / none values
    assert_eq!(xbi_wasm.encode().len(), 61);
}

#[test]
fn custom_encodes_decodes_xbi_call_custom() {
    let xbi_call_custom = XBIInstr::CallCustom {
        caller: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        dest: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        value: 1,
        input: vec![8, 9],
        limit: 1,
        additional_params: vec![10u8, 11u8],
    };

    let decoded_xbi_call_custom: XBIInstr =
        Decode::decode(&mut &xbi_call_custom.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_call_custom.encode(), xbi_call_custom.encode());
    assert_eq!(xbi_call_custom, decoded_xbi_call_custom);
}

#[test]
fn custom_encodes_decodes_empty_xbi_call_custom() {
    let xbi_call_custom = XBIInstr::CallCustom {
        caller: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        dest: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        value: 1,
        input: vec![],
        limit: 1,
        additional_params: vec![],
    };

    let decoded_xbi_call_custom: XBIInstr =
        Decode::decode(&mut &xbi_call_custom.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_call_custom.encode(), xbi_call_custom.encode());
    assert_eq!(xbi_call_custom, decoded_xbi_call_custom);
    assert_eq!(xbi_call_custom.encode().len(), 93);
}

#[test]
fn custom_encodes_decodes_xbi_transfer() {
    let xbi_transfer = XBIInstr::Transfer {
        dest: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        value: 1,
    };

    let decoded_xbi_transfer: XBIInstr = Decode::decode(&mut &xbi_transfer.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_transfer.encode(), xbi_transfer.encode());
    assert_eq!(xbi_transfer, decoded_xbi_transfer);
    assert_eq!(xbi_transfer.encode().len(), 49);
}

#[test]
fn custom_encodes_decodes_xbi_transfer_assets() {
    let xbi_transfer_assets = XBIInstr::TransferAssets {
        currency_id: 1u32,
        dest: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        value: 1,
    };

    let decoded_xbi_transfer_assets: XBIInstr =
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
    let xbi_result = XBIInstr::Result {
        outcome: XBICheckOutStatus::SuccessfullyExecuted,
        output: vec![1, 2, 3],
        witness: vec![4, 5, 6],
    };

    let decoded_xbi_result: XBIInstr = Decode::decode(&mut &xbi_result.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_result.encode(), xbi_result.encode());
    assert_eq!(xbi_result, decoded_xbi_result);
}

#[test]
fn custom_encodes_decodes_xbi_notification() {
    let xbi_notification = XBIInstr::Notification {
        kind: XBINotificationKind::Sent,
        instruction_id: vec![1, 2, 3],
        extra: vec![4, 5, 6],
    };

    let decoded_xbi_notification: XBIInstr =
        Decode::decode(&mut &xbi_notification.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_notification.encode(), xbi_notification.encode());
    assert_eq!(xbi_notification, decoded_xbi_notification);
}
