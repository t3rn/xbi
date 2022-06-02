use codec::{Decode, Encode};
use core::fmt::Debug;
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_runtime::AccountId32;
use sp_std::prelude::*;

use xcm::latest::{Junction, MultiLocation};

/// Basic Types.
/// Could also introduce t3rn-primitives/abi but perhaps easier to align on sp_std types
pub type Bytes = Vec<u8>;
/// Introduce enum vs u32/u64 and cast later?
pub type AssetId = u64;
/// Could be a MultiAsset?
pub type Balance16B = u128;
pub type AccountIdOf = MultiLocation;
// pub type AccountId32 = AccountId32;
// pub type AccountKey20 = AccountKey20;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum XBICheckOutStatus {
    SuccessfullyExecuted,
    ErrorFailedExecution,
    ErrorFailedXCMDispatch,
    ErrorDeliveryTimeout,
    ErrorExecutionTimeout,
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
}

impl XBICheckOut {
    pub fn new<T: frame_system::Config + crate::pallet::Config>(
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
            checkout_timeout: Default::default()
            // FixMe: casting block to timeout
            // checkout_timeout: ((frame_system::Pallet::<T>::block_number() - delivery_timeout)
            //     * T::BlockNumber::from(T::ExpectedBlockTimeMs::get())).into(),
        }
    }

    /// For WASM and EVM to exchange the output result suggest conversions here from PostDispatchResults from EVM -> WASM and WASM -> EVM
    pub fn evm_to_wasm_output<T: frame_system::Config + crate::pallet::Config>(
    ) -> Result<Vec<u8>, crate::Error<T>> {
        Ok(vec![])
    }

    pub fn wasm_to_evm_output<T: frame_system::Config + crate::pallet::Config>(
    ) -> Result<Vec<u8>, crate::Error<T>> {
        Ok(vec![])
    }
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct XBICheckIn<BlockNumber> {
    pub xbi: XBIFormat,
    pub notification_delivery_timeout: BlockNumber,
    pub notification_execution_timeout: BlockNumber,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct XBIFormat {
    pub instr: XBIInstr,
    pub metadata: XBIMetadata,
}
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub enum XBIInstr {
    CallNative {
        payload: Bytes,
    },
    CallEvm {
        caller: AccountId32,
        dest: Junction, // Junction::AccountKey20
        value: Balance16B,
        input: Bytes,
        gas_limit: Balance16B,
        max_fee_per_gas: Option<Balance16B>,
        max_priority_fee_per_gas: Option<Balance16B>,
        nonce: Option<u32>,
        access_list: Option<Bytes>,
    },
    CallWasm {
        caller: AccountId32,
        dest: AccountId32,
        value: Balance16B,
        input: Bytes,
    },
    CallCustom {
        caller: AccountId32,
        dest: AccountId32,
        value: Balance16B,
        input: Bytes,
        additional_params: Option<Vec<Bytes>>,
    },
    Transfer {
        dest: AccountId32,
        value: Balance16B,
    },
    TransferORML {
        currency_id: AssetId,
        dest: AccountId32,
        value: Balance16B,
    },
    TransferAssets {
        currency_id: AssetId,
        dest: AccountId32,
        value: Balance16B,
    },
    Result {
        outcome: XBICheckOutStatus,
        output: Bytes,
        witness: Bytes,
    },
    Notification {
        kind: XBINotificationKind,
        instruction_id: Bytes,
        extra: Bytes,
    },
}

impl Default for XBIInstr {
    fn default() -> Self {
        XBIInstr::CallNative { payload: vec![] }
    }
}

pub type Timeout = u32;

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub enum XBINotificationKind {
    Sent,
    Delivered,
    Executed,
}
#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct ActionNotificationTimeouts {
    pub action: Timeout,
    pub notification: Timeout,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Encode, Decode, TypeInfo)]
pub struct XBIMetadata {
    pub id: sp_core::H256,
    pub dest_para_id: u32,
    pub src_para_id: u32,
    pub sent: ActionNotificationTimeouts,
    pub delivered: ActionNotificationTimeouts,
    pub executed: ActionNotificationTimeouts,
    pub max_exec_cost: Balance16B,
    pub max_notifications_cost: Balance16B,
}

/// max_exec_cost satisfies all of the execution fee requirements while going through XCM execution:
/// max_exec_cost -> exec_in_credit
/// max_exec_cost -> exec_in_credit -> max execution cost (EVM/WASM::max_gas_fees)
impl XBIMetadata {
    pub fn to_exec_in_credit<T: crate::Config, Balance: Encode + Decode + Clone>(
        &self,
    ) -> Result<Balance, crate::Error<T>> {
        Decode::decode(&mut &self.max_notifications_cost.encode()[..])
            .map_err(|_e| crate::Error::<T>::EnterFailedOnMultiLocationTransform)
    }
}
// //   - `Sent (action timeout, notification timeout)`
// //   - `Delivered (action timeout, notification timeout)`
// //   - `Executed (action timeout, notification timeout)`
// //   - `Destination / Bridge security guarantees (e.g. in confirmation no for PoW, finality proofs)`
// //   - `max_exec_cost`: `Balance` : `Maximal cost / fees for execution of delivery`
// //   - `max_notification_cost`: `Balance` : `Maximal cost / fees per delivering notification`
//
// pub enum XBIMetadata {
// 	Sent { action: Timeout, notification: Timeout },
// 	Delivered { action: Timeout, notification: Timeout },
// 	Executed { action: Timeout, notification: Timeout },
// 	// //   - `Sent (action timeout, notification timeout)`
// 	// //   - `Delivered (action timeout, notification timeout)`
// 	// //   - `Executed (action timeout, notification timeout)`
// 	// //   - `Destination / Bridge security guarantees (e.g. in confirmation no for PoW, finality proofs)`
// 	// //   - `max_exec_cost`: `Balance` : `Maximal cost / fees for execution of delivery`
// 	// //   - `max_notification_cost`: `Balance` : `Maximal cost / fees per delivering notification`
// }
