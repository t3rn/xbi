use crate::{xbi_abi::*, xbi_format::XBIInstr};

use crate::xbi_format::{XBICheckOutStatus, XBINotificationKind};
use codec::{Decode, Encode};

#[test]
fn custom_encodes_decodes_xbi_evm() {
    let xbi_evm = XBIInstr::CallEvm {
        source: AccountId20::repeat_byte(3),
        dest: AccountId20::repeat_byte(2),
        value: 1,
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
fn custom_encodes_decodes_empty_xbi_evm() {
    let xbi_evm = XBIInstr::CallEvm {
        source: AccountId20::repeat_byte(3),
        dest: AccountId20::repeat_byte(2),
        value: 1,
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
    assert_eq!(xbi_evm.encode().len(), 105);
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
        additional_params: vec![],
    };

    let decoded_xbi_call_custom: XBIInstr =
        Decode::decode(&mut &xbi_call_custom.encode()[..]).unwrap();
    assert_eq!(decoded_xbi_call_custom.encode(), xbi_call_custom.encode());
    assert_eq!(xbi_call_custom, decoded_xbi_call_custom);
    assert_eq!(xbi_call_custom.encode().len(), 85);
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
fn custom_encodes_decodes_xbi_transfer_orml() {
    let xbi_transfer_orml = XBIInstr::TransferORML {
        currency_id: 1u64,
        dest: AccountId32::new([
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ]),
        value: 1,
    };

    let decoded_xbi_transfer_orml: XBIInstr =
        Decode::decode(&mut &xbi_transfer_orml.encode()[..]).unwrap();
    assert_eq!(
        decoded_xbi_transfer_orml.encode(),
        xbi_transfer_orml.encode()
    );
    assert_eq!(xbi_transfer_orml, decoded_xbi_transfer_orml);

    assert_eq!(xbi_transfer_orml.encode().len(), 57);
}

#[test]
fn custom_encodes_decodes_xbi_transfer_assets() {
    let xbi_transfer_assets = XBIInstr::TransferAssets {
        currency_id: 1u64,
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
    assert_eq!(xbi_transfer_assets.encode().len(), 57);
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
