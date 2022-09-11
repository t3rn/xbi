use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_runtime::AccountId32;
use sp_std::prelude::*;

pub mod xbi_codec;

use sabi::*;
pub use xbi_codec::*;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum XBICheckOutStatus {
    // Success scenario
    SuccessfullyExecuted,

    // Failed execution scenarios
    ErrorFailedExecution,
    ErrorFailedOnXCMDispatch,

    // Failed with exceeded costs scenarios
    ErrorExecutionCostsExceededAllowedMax,
    ErrorNotificationsCostsExceededAllowedMax,

    // Failed with exceeded timeout scenarios
    ErrorSentTimeoutExceeded,
    ErrorDeliveryTimeoutExceeded,
    ErrorExecutionTimeoutExceeded,
}

impl Default for XBICheckOutStatus {
    fn default() -> Self {
        XBICheckOutStatus::SuccessfullyExecuted
    }
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct XBICheckOut {
    pub xbi: XBIInstr, // XBIInstr::Result
    pub resolution_status: XBICheckOutStatus,
    pub checkout_timeout: Timeout,
    pub actual_execution_cost: Value,
    pub actual_delivery_cost: Value,
    pub actual_aggregated_cost: Value,
}

impl XBICheckOut {
    pub fn new<T: frame_system::Config>(
        _delivery_timeout: T::BlockNumber,
        output: Vec<u8>,
        resolution_status: XBICheckOutStatus,
        actual_execution_cost: Value,
        actual_delivery_cost: Value,
        actual_aggregated_cost: Value,
    ) -> Self {
        XBICheckOut {
            xbi: XBIInstr::Result {
                outcome: resolution_status.clone(),
                output,
                witness: vec![],
            },
            resolution_status,
            checkout_timeout: Default::default(), // FixMe: make below work - casting block no to timeout
            // checkout_timeout: ((frame_system::Pallet::<T>::block_number() - delivery_timeout)
            //     * T::BlockNumber::from(T::ExpectedBlockTimeMs::get())).into(),
            actual_execution_cost,
            actual_delivery_cost,
            actual_aggregated_cost,
        }
    }

    pub fn new_ignore_costs<T: frame_system::Config>(
        _delivery_timeout: T::BlockNumber,
        output: Vec<u8>,
        resolution_status: XBICheckOutStatus,
    ) -> Self {
        XBICheckOut {
            xbi: XBIInstr::Result {
                outcome: resolution_status.clone(),
                output,
                witness: vec![],
            },
            resolution_status,
            checkout_timeout: Default::default(), // FixMe: make below work - casting block no to timeout
            // checkout_timeout: ((frame_system::Pallet::<T>::block_number() - delivery_timeout)
            //     * T::BlockNumber::from(T::ExpectedBlockTimeMs::get())).into(),
            actual_execution_cost: 0,
            actual_delivery_cost: 0,
            actual_aggregated_cost: 0,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct XBICheckIn<BlockNumber> {
    pub xbi: XBIFormat,
    pub notification_delivery_timeout: BlockNumber,
    pub notification_execution_timeout: BlockNumber,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Default, Encode, Decode, TypeInfo)]
pub struct XBIFormat {
    pub instr: XBIInstr,
    pub metadata: XBIMetadata,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum XBIInstr {
    // 0
    Unknown {
        identifier: u8,
        params: Vec<u8>,
    },
    // 1
    CallNative {
        payload: Data,
    },
    // 2
    CallEvm {
        source: AccountId20, // Could use either [u8; 20] or Junction::AccountKey20
        target: AccountId20, // Could use either [u8; 20] or Junction::AccountKey20
        value: ValueEvm,
        input: Data,
        gas_limit: Gas,
        max_fee_per_gas: ValueEvm,
        max_priority_fee_per_gas: Option<ValueEvm>,
        nonce: Option<ValueEvm>,
        access_list: Vec<(AccountId20, Vec<sp_core::H256>)>, // Could use Vec<([u8; 20], Vec<[u8; 32]>)>,
    },
    // 3
    CallWasm {
        dest: AccountId32,
        value: Value,
        gas_limit: Gas,
        storage_deposit_limit: Option<Value>,
        data: Data,
    },
    // 4
    CallCustom {
        caller: AccountId32,
        dest: AccountId32,
        value: Value,
        input: Data,
        limit: Gas,
        additional_params: Data,
    },
    // 5
    Transfer {
        dest: AccountId32,
        value: Value,
    },
    // 6
    TransferAssets {
        currency_id: AssetId,
        dest: AccountId32,
        value: Value,
    },
    // 7
    Swap {
        asset_out: AssetId,
        asset_in: AssetId,
        amount: Value,
        max_limit: Value,
        discount: bool,
    },
    // 8
    AddLiquidity {
        asset_a: AssetId,
        asset_b: AssetId,
        amount_a: Value,
        amount_b_max_limit: Value,
    },
    // 9
    RemoveLiquidity {
        asset_a: AssetId,
        asset_b: AssetId,
        liquidity_amount: Value,
    },
    // 10
    GetPrice {
        asset_a: AssetId,
        asset_b: AssetId,
        amount: Value,
    },
    // 255
    Result {
        outcome: XBICheckOutStatus,
        output: Data,
        witness: Data,
    },
}

impl Default for XBIInstr {
    fn default() -> Self {
        XBIInstr::CallNative { payload: vec![] }
    }
}

pub type Timeout = u32;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub enum XBINotificationKind {
    Sent,
    Delivered,
    Executed,
}
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct ActionNotificationTimeouts {
    pub action: Timeout,
    pub notification: Timeout,
}

impl Default for ActionNotificationTimeouts {
    fn default() -> Self {
        ActionNotificationTimeouts {
            action: 96000,       // 96 sec
            notification: 24000, // 24 sec
        }
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Default, Encode, Decode, TypeInfo)]
pub struct XBIMetadata {
    pub id: sp_core::H256,
    pub dest_para_id: u32,
    pub src_para_id: u32,
    pub sent: ActionNotificationTimeouts,
    pub delivered: ActionNotificationTimeouts,
    pub executed: ActionNotificationTimeouts,
    pub max_exec_cost: Value,
    pub max_notifications_cost: Value,
    pub actual_aggregated_cost: Option<Value>,
    pub maybe_known_origin: Option<AccountId32>,
    pub maybe_fee_asset_id: Option<AssetId>,
}

/// max_exec_cost satisfies all of the execution fee requirements while going through XCM execution:
/// max_exec_cost -> exec_in_credit
/// max_exec_cost -> exec_in_credit -> max execution cost (EVM/WASM::max_gas_fees)
impl XBIMetadata {
    pub fn new(
        id: sp_core::H256,
        dest_para_id: u32,
        src_para_id: u32,
        sent: ActionNotificationTimeouts,
        delivered: ActionNotificationTimeouts,
        executed: ActionNotificationTimeouts,
        max_exec_cost: Value,
        max_notifications_cost: Value,
        maybe_known_origin: Option<AccountId32>,
        maybe_fee_asset_id: Option<AssetId>,
    ) -> Self {
        XBIMetadata {
            id,
            dest_para_id,
            src_para_id,
            sent,
            delivered,
            executed,
            max_exec_cost,
            max_notifications_cost,
            maybe_known_origin,
            actual_aggregated_cost: None,
            maybe_fee_asset_id,
        }
    }

    pub fn new_with_default_timeouts(
        id: sp_core::H256,
        dest_para_id: u32,
        src_para_id: u32,
        max_exec_cost: Value,
        max_notifications_cost: Value,
        maybe_known_origin: Option<AccountId32>,
        maybe_fee_asset_id: Option<AssetId>,
    ) -> Self {
        XBIMetadata {
            id,
            dest_para_id,
            src_para_id,
            sent: ActionNotificationTimeouts::default(),
            delivered: ActionNotificationTimeouts::default(),
            executed: ActionNotificationTimeouts::default(),
            max_exec_cost,
            max_notifications_cost,
            maybe_known_origin,
            actual_aggregated_cost: None,
            maybe_fee_asset_id,
        }
    }
}
