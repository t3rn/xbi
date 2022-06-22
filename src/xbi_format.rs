use codec::{Decode, Encode, Input, Output};
use core::fmt::Debug;
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_runtime::AccountId32;
use sp_std::prelude::*;

pub use crate::{xbi_abi::*, xbi_codec::*};

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
            // FixMe: make below work - casting block no to timeout
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

//pub fn call_xbi(instruction:  ) {
// pub fn call_xbi(instruction: Vec<u8>) {
//     XBIInstr::decode(&mut &*instruction);
//
// }

#[derive(Clone, Eq, PartialEq, Debug, TypeInfo)]
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
        dest: AccountId20,   // Could use either [u8; 20] or Junction::AccountKey20
        value: Value,
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
        additional_params: Data,
    },
    Transfer {
        dest: AccountId32,
        value: Value,
    },
    TransferORML {
        currency_id: AssetId,
        dest: AccountId32,
        value: Value,
    },
    TransferAssets {
        currency_id: AssetId,
        dest: AccountId32,
        value: Value,
    },
    Result {
        outcome: XBICheckOutStatus,
        output: Data,
        witness: Data,
    },
    // 9
    Notification {
        kind: XBINotificationKind,
        instruction_id: Data,
        extra: Data,
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
    pub max_exec_cost: Value,
    pub max_notifications_cost: Value,
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
