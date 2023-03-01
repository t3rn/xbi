#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, FullCodec};
use core::fmt::Debug;
use scale_info::TypeInfo;
use sp_core::Hasher;
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_runtime::AccountId32;
use sp_std::prelude::*;
use sp_std::vec;

pub mod xbi_codec;

use sabi::*;
pub use xbi_codec::*;

/// A representation of the status of an XBI execution
#[derive(Clone, Eq, PartialEq, Default, Encode, Decode, Debug, TypeInfo)]
pub enum Status {
    #[default]
    /// The XBI message was successful
    Success,
    /// Failed to execute an XBI instruction
    FailedExecution,
    /// An error occurred whilst sending the XCM message
    DispatchFailed,
    /// The execution exceeded the maximum cost provided in the message
    ExecutionLimitExceeded,
    /// The notification cost for the message was exceeded
    NotificationLimitExceeded,
    /// The XBI reqeuest timed out when trying to dispatch the message
    SendTimeout,
    /// The XBI request timed out before the message was received by the target
    DeliveryTimeout,
    /// The message timed out before the execution occured on the target
    ExecutionTimeout,
}

impl From<&Fees> for Status {
    fn from(value: &Fees) -> Self {
        if value.notification_limit_exceeded() {
            Status::NotificationLimitExceeded
        } else if value.execution_limit_exceeded() {
            Status::ExecutionLimitExceeded
        } else {
            Status::Success
        }
    }
}

// TODO: enrich the dtos in this library with reusable structs, so that the data is not flat.
// e.g parachain dest & source are the same, but described by the variable over some dto.

/// A representation of the state of an XBI message, meters relating to cost and any timeouts
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, Debug, TypeInfo)]
pub struct XbiCheckOut {
    /// The requested instruction
    pub xbi: XbiInstruction, // TODO: XbiInstruction::Result(XbiResult { }), then the result can be a struct here
    /// The status of the message
    pub resolution_status: Status,
    pub checkout_timeout: Timeout,
    /// The metered cost of the message to be handled
    pub actual_execution_cost: Value,
    /// The cost to send the message
    pub actual_delivery_cost: Value,
    // TODO: this can be calculated by a function on XbiCheckout
    /// The cost of the message with the execution cost
    pub actual_aggregated_cost: Value,
}

/// An XBI message with additional timeout information
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, Debug, TypeInfo)]
pub struct XbiCheckIn<BlockNumber> {
    /// The XBI message
    pub xbi: XbiFormat,
    /// Timeout information for checking the queue
    pub notification_delivery_timeout: BlockNumber,
    /// Timeout information for checking the result of the execution
    pub notification_execution_timeout: BlockNumber,
}

/// An XBI message
#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct XbiFormat {
    /// The instruction to execute on the target
    pub instr: XbiInstruction,
    /// Additional information about the target, costs and any user defined timeouts relating to the message
    pub metadata: XbiMetadata,
}

// TODO: implement into<usize> to specify custom, versioned byte representations. E.g Result = 255
/// The instruction to execute on the target
#[derive(Clone, Eq, PartialEq, Debug, TypeInfo)]
pub enum XbiInstruction {
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
}

impl Default for XbiInstruction {
    fn default() -> Self {
        XbiInstruction::CallNative { payload: vec![] }
    }
}

/// A result containing the status of the call
#[derive(Debug, Clone, Eq, Default, PartialEq, Encode, Decode, TypeInfo)]
pub struct XbiResult {
    pub status: Status,
    pub output: Data,
    pub witness: Data,
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

const TIMEOUT_NINETY_SIX_SECONDS: Timeout = 96000;
const TIMEOUT_TWENTY_FOUR_SECONDS: Timeout = 24000;

impl Default for ActionNotificationTimeouts {
    fn default() -> Self {
        ActionNotificationTimeouts {
            action: TIMEOUT_NINETY_SIX_SECONDS,        // 96 sec
            notification: TIMEOUT_TWENTY_FOUR_SECONDS, // 24 sec
        }
    }
}

// TODO: add requested here so we can time when the message was sent
#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct Timeouts {
    /// Timeouts in relation to when the message should be sent
    pub sent: ActionNotificationTimeouts,
    /// Timeouts in relation to when the message should be delivered
    pub delivered: ActionNotificationTimeouts,
    /// Timeouts in relation to when the message should be executed
    pub executed: ActionNotificationTimeouts,
    /// Timeouts in relation to when the message should be responded
    pub responded: ActionNotificationTimeouts,
}

impl Timeouts {
    pub fn new(
        sent: Option<ActionNotificationTimeouts>,
        delivered: Option<ActionNotificationTimeouts>,
        executed: Option<ActionNotificationTimeouts>,
        responded: Option<ActionNotificationTimeouts>,
    ) -> Self {
        Self {
            sent: sent.unwrap_or_default(),
            delivered: delivered.unwrap_or_default(),
            executed: executed.unwrap_or_default(),
            responded: responded.unwrap_or_default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct Fees {
    /// The asset to pay the fees in, otherwise native
    pub asset: Option<AssetId>,
    /// The maximum cost of the execution of the message
    pub execution_cost_limit: Value,
    /// The maximum cost of sending any notifications
    pub notification_cost_limit: Value,
    /// The cost of execution and notification
    aggregated_cost: Value,
}

impl Fees {
    pub fn new(
        asset: Option<AssetId>,
        max_exec_cost: Option<Value>,
        max_notifications_cost: Option<Value>,
    ) -> Self {
        Fees {
            asset,
            execution_cost_limit: max_exec_cost.unwrap_or_default(),
            notification_cost_limit: max_notifications_cost.unwrap_or_default(),
            aggregated_cost: Value::default(),
        }
    }

    pub fn push_aggregate(&mut self, value: Value) {
        if let Some(new) = self.aggregated_cost.checked_add(value) {
            self.aggregated_cost = new;
        } else {
            log::warn!("Overflow when adding to aggregate cost");
        }
    }

    pub fn get_aggregated_limit(&self) -> Value {
        self.execution_cost_limit + self.notification_cost_limit
    }

    pub fn limit_exceeded(&self) -> bool {
        self.aggregated_cost > self.get_aggregated_limit()
    }

    pub fn execution_limit_exceeded(&self) -> bool {
        self.aggregated_cost > self.execution_cost_limit
    }

    pub fn notification_limit_exceeded(&self) -> bool {
        self.aggregated_cost > self.notification_cost_limit
    }

    pub fn get_aggregated_cost(&self) -> Value {
        self.aggregated_cost
    }
}

// TODO: use enum instead
/// Timesheet Analogous to XbiMetadata::timeouts but tracked onchain
///
/// Utilised by the queue to determine when to stop progressing an item
#[derive(Debug, Clone, PartialEq, Default, Eq, Encode, Decode, TypeInfo)]
pub struct XbiTimeSheet<BlockNumber: FullCodec + TypeInfo> {
    /// At the time of user submission
    pub submitted: Option<BlockNumber>,
    /// When sent over xcm
    pub sent: Option<BlockNumber>,
    /// When received by the receiver
    pub delivered: Option<BlockNumber>,
    /// When executed
    pub executed: Option<BlockNumber>,
    /// When the response was received
    pub responded: Option<BlockNumber>,
}

pub enum Timestamp<BlockNumber: FullCodec + TypeInfo> {
    Submitted(BlockNumber),
    Sent(BlockNumber),
    Delivered(BlockNumber),
    Executed(BlockNumber),
    Responded(BlockNumber),
}

impl<BlockNumber: FullCodec + TypeInfo> XbiTimeSheet<BlockNumber> {
    pub fn new() -> Self {
        XbiTimeSheet {
            submitted: None,
            sent: None,
            delivered: None,
            executed: None,
            responded: None,
        }
    }

    pub fn progress(&mut self, timestamp: Timestamp<BlockNumber>) -> &mut Self {
        match timestamp {
            Timestamp::Submitted(block) => self.submitted = Some(block),
            Timestamp::Sent(block) => self.sent = Some(block),
            Timestamp::Delivered(block) => self.delivered = Some(block),
            Timestamp::Executed(block) => self.executed = Some(block),
            Timestamp::Responded(block) => self.responded = Some(block),
        }
        self
    }
}

/// Additional information about the target, costs and any user defined timeouts relating to the message
#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct XbiMetadata {
    /// The XBI identifier
    id: sp_core::H256,
    /// The destination parachain
    pub dest_para_id: u32,
    /// The src parachain
    pub src_para_id: u32,
    /// User provided timeouts
    pub timeouts: Timeouts,
    /// The time sheet providing timestamps to each of the xbi progression
    timesheet: XbiTimeSheet<u32>, // TODO: assume u32 is block number
    /// User provided cost limits
    pub fees: Fees,
    /// The optional known caller
    pub maybe_known_origin: Option<AccountId32>,
}

/// max_exec_cost satisfies all of the execution fee requirements while going through XCM execution:
/// max_exec_cost -> exec_in_credit
/// max_exec_cost -> exec_in_credit -> max execution cost (EVM/WASM::max_gas_fees)
// TODO: implement builders for XBI metadata fields
impl XbiMetadata {
    // TODO: feature flag for this, uncouple, just pass traits
    // pub fn to_exec_in_credit<T: crate::Config, Balance: Encode + Decode + Clone>(
    //     &self,
    // ) -> Result<Balance, crate::Error<T>> {
    //     Decode::decode(&mut &self.max_notifications_cost.encode()[..])
    //         .map_err(|_e| crate::Error::<T>::EnterFailedOnMultiLocationTransform)
    // }

    /// Provide a hash of the XBI msg id
    pub fn rehash_id<Hashing: Hash + Hasher<Out = <Hashing as Hash>::Output>>(
        &self,
    ) -> <Hashing as Hasher>::Out {
        <Hashing as Hasher>::hash(&self.id.encode()[..])
    }

    /// Provide the hashable fields for the metadata to retrieve the id, this omits the things that are normally different across the lifecycle of the message
    pub(crate) fn sane_fields(&self) -> Vec<Vec<u8>> {
        vec![
            self.src_para_id.encode(),
            self.dest_para_id.encode(),
            self.timeouts.encode(),
            self.fees.asset.encode(),
            self.fees.execution_cost_limit.encode(),
            self.fees.notification_cost_limit.encode(),
            self.maybe_known_origin.encode(),
        ]
    }

    pub fn sane_hashable_fields(&self) -> Vec<u8> {
        self.sane_fields().concat()
    }

    pub fn enrich_id<Hashing: Hasher<Out = sp_core::H256>>(
        &mut self,
        nonce: u32,
        seed: Option<&[u8]>,
    ) -> &mut Self {
        if self.id == sp_core::H256::default() {
            let mut hash_contents = vec![nonce.encode(), self.sane_hashable_fields()];
            if let Some(seed) = seed {
                hash_contents.push(seed.to_vec());
            }

            self.id = Hashing::hash(&hash_contents.concat()[..]);
        } else {
            log::warn!("Can only enrich the id if it has not been enriched already");
        }
        self
    }

    pub fn get_id(&self) -> sp_core::H256 {
        self.id
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        src_para_id: u32,
        dest_para_id: u32,
        timeouts: Timeouts,
        fees: Fees,
        maybe_known_origin: Option<AccountId32>,
        nonce: u32,
        seed: Option<&[u8]>,
    ) -> Self {
        let mut base = XbiMetadata {
            id: Default::default(),
            dest_para_id,
            src_para_id,
            timeouts,
            timesheet: Default::default(),
            fees,
            maybe_known_origin,
        };
        base.enrich_id::<BlakeTwo256>(nonce, seed).to_owned()
    }

    pub fn progress(&mut self, timestamp: Timestamp<u32>) -> &mut Self {
        self.timesheet.progress(timestamp);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Timestamp::*;
    use sp_runtime::traits::BlakeTwo256;

    #[test]
    fn can_hash_id() {
        let meta = XbiMetadata::default();
        let hash = meta.rehash_id::<BlakeTwo256>();
        assert_eq!(hash, <BlakeTwo256 as Hasher>::hash(&meta.id.0))
    }

    #[test]
    fn timesheet_can_progress() {
        let mut timesheet = XbiTimeSheet::<u32>::new();

        timesheet.progress(Submitted(1));
        timesheet.progress(Sent(2));
        assert_eq!(timesheet.submitted, Some(1));
        assert_eq!(timesheet.sent, Some(2));

        timesheet.progress(Delivered(3));
        assert_eq!(timesheet.submitted, Some(1));
        assert_eq!(timesheet.sent, Some(2));
        assert_eq!(timesheet.delivered, Some(3));

        timesheet.progress(Executed(4));
        assert_eq!(timesheet.submitted, Some(1));
        assert_eq!(timesheet.sent, Some(2));
        assert_eq!(timesheet.delivered, Some(3));
        assert_eq!(timesheet.executed, Some(4));

        timesheet.progress(Responded(5));
        assert_eq!(timesheet.submitted, Some(1));
        assert_eq!(timesheet.sent, Some(2));
        assert_eq!(timesheet.delivered, Some(3));
        assert_eq!(timesheet.executed, Some(4));
        assert_eq!(timesheet.responded, Some(5));
    }

    #[test]
    fn push_aggregate_works() {
        let mut fees = Fees {
            asset: None,
            execution_cost_limit: 0,
            notification_cost_limit: 0,
            aggregated_cost: 0,
        };

        fees.push_aggregate(100);

        assert_eq!(fees.aggregated_cost, 100);
    }

    #[test]
    fn test_status() {
        let mut fees = Fees {
            asset: None,
            execution_cost_limit: 1,
            notification_cost_limit: 1,
            aggregated_cost: 1,
        };
        assert_eq!(Status::from(&fees), Status::Success);

        fees.execution_cost_limit = 0;
        assert_eq!(Status::from(&fees), Status::ExecutionLimitExceeded);

        fees.notification_cost_limit = 0;
        assert_eq!(Status::from(&fees), Status::NotificationLimitExceeded);
    }

    #[test]
    fn test_can_enrich_id() {
        let mut meta = XbiMetadata::default();
        let nonce = 1;

        meta.enrich_id::<sp_runtime::traits::BlakeTwo256>(nonce, None);
        assert_ne!(meta.id, sp_core::H256::default());
        let expected_fields = vec![nonce.encode(), meta.sane_hashable_fields()].concat();
        assert_eq!(
            meta.id,
            <sp_runtime::traits::BlakeTwo256 as sp_core::Hasher>::hash(&expected_fields[..])
        );
    }

    // test that the sane_hashable fields do not contain the insane fields
    #[test]
    fn test_sane_hashable_fields() {
        let meta = XbiMetadata::default();
        let sane_fields = meta.sane_fields();
        assert!(!sane_fields.contains(&meta.id.encode()));
        assert!(!sane_fields.contains(&meta.timesheet.encode()));
    }
}
