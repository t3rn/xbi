use codec::{Decode, Encode};
use core::fmt::Debug;
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_core::Hasher;
use sp_runtime::traits::Hash;
use sp_runtime::AccountId32;
use sp_std::prelude::*;

pub mod xbi_codec;

use sabi::*;
pub use xbi_codec::*;

/// A representation of the status of an XBI execution
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum XBICheckOutStatus {
    /// The XBI message was successful
    SuccessfullyExecuted,
    /// Failed to execute an XBI instruction
    ErrorFailedExecution,
    /// An error occurred whilst sending the XCM message
    ErrorFailedOnXCMDispatch,
    /// The execution exceeded the maximum cost provided in the message
    ErrorExecutionCostsExceededAllowedMax,
    /// The notification cost for the message was exceeded
    ErrorNotificationsCostsExceededAllowedMax,
    /// The XBI reqeuest timed out when trying to dispatch the message
    ErrorSentTimeoutExceeded,
    /// The XBI request timed out before the message was received by the target
    ErrorDeliveryTimeoutExceeded,
    /// The message timed out before the execution occured on the target
    ErrorExecutionTimeoutExceeded,
}

impl Default for XBICheckOutStatus {
    fn default() -> Self {
        XBICheckOutStatus::SuccessfullyExecuted
    }
}

// TODO: enrich the dtos in this library with reusable structs, so that the data is not flat.
// e.g parachain dest & source are the same, but described by the variable over some dto.

/// A representation of the state of an XBI message, meters relating to cost and any timeouts
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct XBICheckOut {
    /// The requested instruction
    pub xbi: XBIInstr, // TODO: XBIInstr::Result(XbiResult { }), then the result can be a struct here
    /// The status of the message
    pub resolution_status: XBICheckOutStatus,
    pub checkout_timeout: Timeout,
    /// The metered cost of the message to be handled
    pub actual_execution_cost: Value,
    /// The cost to send the message
    pub actual_delivery_cost: Value,
    // TODO: this can be calculated by a function on XBICheckout
    /// The cost of the message with the execution cost
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
            checkout_timeout: Default::default(),
            // fixme: make below work - casting block no to timeout
            // provide some differential function so not to couple to `frame`
            // checkout_timeout: ((frame_system::Pallet::<T>::block_number() - delivery_timeout)
            //     * T::BlockNumber::from(T::ExpectedBlockTimeMs::get())).into(),
            actual_execution_cost,
            actual_delivery_cost,
            actual_aggregated_cost,
        }
    }

    /// Instantiate a new checkout with default costs
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
            checkout_timeout: Default::default(),
            // fixme: make below work - casting block no to timeout
            // provide some differential function so not to couple to `frame`
            //     * T::BlockNumber::from(T::ExpectedBlockTimeMs::get())).into(),
            actual_execution_cost: 0,
            actual_delivery_cost: 0,
            actual_aggregated_cost: 0,
        }
    }
}

/// An XBI message with additional timeout information
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct XBICheckIn<BlockNumber> {
    /// The XBI message
    pub xbi: XBIFormat,
    /// Timeout information for checking the queue
    pub notification_delivery_timeout: BlockNumber,
    /// Timeout information for checking the result of the execution
    pub notification_execution_timeout: BlockNumber,
}

/// An XBI message
#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct XBIFormat {
    /// The instruction to execute on the target
    pub instr: XBIInstr,
    /// Additional information about the target, costs and any user defined timeouts relating to the message
    pub metadata: XBIMetadata,
}

// TODO: implement into<usize> to specify custom, versioned byte representations. E.g Result = 255
/// The instruction to execute on the target
#[derive(Clone, Eq, PartialEq, Debug, TypeInfo)]
pub enum XBIInstr {
    /// An opaque message providing the instruction identifier and some bytes
    Unknown { identifier: u8, params: Vec<u8> },
    /// A call native to the parachain, this is also opaque and can be custom
    CallNative { payload: Data },
    /// A call to an EVM contract
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
    /// A call to a WASM contract
    CallWasm {
        dest: AccountId32,
        value: Value,
        gas_limit: Gas,
        storage_deposit_limit: Option<Value>,
        data: Data,
    },
    /// A call to any other VM
    CallCustom {
        caller: AccountId32,
        dest: AccountId32,
        value: Value,
        input: Data,
        limit: Gas,
        additional_params: Data,
    },
    /// A simple transfer
    Transfer { dest: AccountId32, value: Value },
    /// A multiple asset transfer
    TransferAssets {
        currency_id: AssetId,
        dest: AccountId32,
        value: Value,
    },
    /// A DeFi swap
    Swap {
        asset_out: AssetId,
        asset_in: AssetId,
        amount: Value,
        max_limit: Value,
        discount: bool,
    },
    /// A DeFi Add liquidity instruction
    AddLiquidity {
        asset_a: AssetId,
        asset_b: AssetId,
        amount_a: Value,
        amount_b_max_limit: Value,
    },
    /// A DeFi Remove liquidity instruction
    RemoveLiquidity {
        asset_a: AssetId,
        asset_b: AssetId,
        liquidity_amount: Value,
    },
    /// Get the price of a given asset A over asset B
    GetPrice {
        asset_a: AssetId,
        asset_b: AssetId,
        amount: Value,
    },
    /// Provide the result of an XBI instruction
    // TODO: make this a tuple type with a struct XbiResult since this would be easier to send back
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

/// A type of notification emitted from XBI
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub enum XBINotificationKind {
    Sent,
    Delivered,
    Executed,
}

pub type Timeout = u32;

/// A user specified timeout for the message, denoting when the action should happen, and any tolerance
/// to when the message should be notified
// TODO: be specific on the unit of time, allow it to be specified
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo)]
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

/// Additional information about the target, costs and any user defined timeouts relating to the message
#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct XBIMetadata {
    /// The XBI identifier
    pub id: sp_core::H256,
    /// The destination parachain
    pub dest_para_id: u32,
    /// The src parachain
    pub src_para_id: u32,
    // TODO: make this nested in a field: timeouts
    /// Timeouts in relation to when the message should be sent
    pub sent: ActionNotificationTimeouts,
    /// Timeouts in relation to when the message should be delivered
    pub delivered: ActionNotificationTimeouts,
    /// Timeouts in relation to when the message should be executed
    pub executed: ActionNotificationTimeouts,
    // TODO: move this to field: costs
    /// The maximum cost of the execution of the message
    pub max_exec_cost: Value,
    /// The maximum cost of sending any notifications
    pub max_notifications_cost: Value,
    /// The cost of execution and notification
    pub actual_aggregated_cost: Option<Value>,
    /// The optional known caller
    pub maybe_known_origin: Option<AccountId32>,
    /// The optional known caller
    pub maybe_fee_asset_id: Option<AssetId>,
}

/// max_exec_cost satisfies all of the execution fee requirements while going through XCM execution:
/// max_exec_cost -> exec_in_credit
/// max_exec_cost -> exec_in_credit -> max execution cost (EVM/WASM::max_gas_fees)
// TODO: implement builders for XBI metadata fields
impl XBIMetadata {
    // TODO: feature flag for this, uncouple, just pass traits
    // pub fn to_exec_in_credit<T: crate::Config, Balance: Encode + Decode + Clone>(
    //     &self,
    // ) -> Result<Balance, crate::Error<T>> {
    //     Decode::decode(&mut &self.max_notifications_cost.encode()[..])
    //         .map_err(|_e| crate::Error::<T>::EnterFailedOnMultiLocationTransform)
    // }

    /// Provide a hash of the XBI msg id
    pub fn id<Hashing: Hash + Hasher<Out = <Hashing as Hash>::Output>>(
        &self,
    ) -> <Hashing as Hasher>::Out {
        <Hashing as Hasher>::hash(&self.id.encode()[..])
    }

    #[allow(clippy::too_many_arguments)]
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
