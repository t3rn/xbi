use codec::Encode;
use frame_support::{dispatch::PostDispatchInfo, weights::Weight};
use frame_system::pallet_prelude::OriginFor;
use sp_core::{H160, H256, U256};
use sp_std::vec::Vec;

use crate::{xbi_abi::*, xbi_format::*, Error};

pub trait Scabi<T: frame_system::Config + pallet_balances::Config> {
    fn args_evm_2_xbi_call_evm(
        origin: OriginFor<T>,
        source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>>;

    fn args_wasm_2_xbi_call_wasm(
        origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        debug: bool,
    ) -> Result<XBIInstr, crate::Error<T>>;

    fn args_evm_2_xbi_call_wasm(
        origin: OriginFor<T>,
        source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>>;

    fn args_wasm_2_xbi_evm_wasm(
        origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        debug: bool,
    ) -> Result<XBIInstr, crate::Error<T>>;

    fn post_dispatch_info_2_xbi_checkout(
        post_dispatch_info: PostDispatchInfo,
        notification_delivery_timeout: T::BlockNumber,
        resolution_status: XBICheckOutStatus,
        actual_delivery_cost: Value,
    ) -> Result<XBICheckOut, Error<T>>;

    fn xbi_checkout_2_post_dispatch_info(
        xbi_checkout: XBICheckOut,
    ) -> Result<PostDispatchInfo, Error<T>>;
}

impl<T: crate::Config + frame_system::Config + pallet_balances::Config> Scabi<T> for XbiAbi<T> {
    fn args_evm_2_xbi_call_evm(
        _origin: OriginFor<T>,
        source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: U256,
        max_priority_fee_per_gas: Option<U256>,
        nonce: Option<U256>,
        access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>> {
        Ok(XBIInstr::CallEvm {
            source,
            target,
            value,
            input,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            nonce,
            access_list,
        })
    }

    fn args_wasm_2_xbi_call_wasm(
        _origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        _debug: bool,
    ) -> Result<XBIInstr, Error<T>> {
        Ok(XBIInstr::CallWasm {
            dest: XbiAbi::account_local_2_global_32(dest)?,
            value: XbiAbi::value_local_2_global(value)?,
            gas_limit,
            storage_deposit_limit: XbiAbi::maybe_value_local_2_maybe_global(storage_deposit_limit)?,
            data,
        })
    }

    /// Use in EVM precompiles / contract to auto-convert the self.call/delegate_call into args to WASM
    fn args_evm_2_xbi_call_wasm(
        _origin: OriginFor<T>,
        _source: H160,
        target: H160,
        input: Vec<u8>,
        value: U256,
        gas_limit: u64,
        _max_fee_per_gas: U256,
        _max_priority_fee_per_gas: Option<U256>,
        _nonce: Option<U256>,
        _access_list: Vec<(H160, Vec<H256>)>,
    ) -> Result<XBIInstr, crate::Error<T>> {
        Ok(XBIInstr::CallWasm {
            dest: crate::sabi::Sabi::account_20_2_account_32(
                target,
                &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            )
            .map_err(|_| crate::Error::XBIABIFailedToCastBetweenTypesAddress)?,
            value: crate::sabi::Sabi::value_256_2_value_128(value)
                .map_err(|_| crate::Error::XBIABIFailedToCastBetweenTypesValue)?,
            gas_limit,
            storage_deposit_limit: None,
            data: input,
        })
    }

    fn args_wasm_2_xbi_evm_wasm(
        origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        _debug: bool,
    ) -> Result<XBIInstr, Error<T>> {
        Ok(XBIInstr::CallEvm {
            source: XbiAbi::account_local_2_global_20(origin)?,
            target: XbiAbi::account_local_2_global_20(dest)?,
            value: XbiAbi::value_local_2_global_evm(value)?,
            input: data,
            gas_limit,
            // Hacky :( - take max_fee_per_gas from storage_deposit_limit?
            max_fee_per_gas: XbiAbi::maybe_value_local_2_global_evm(storage_deposit_limit)?,
            max_priority_fee_per_gas: None,
            nonce: None,
            access_list: vec![],
        })
    }

    fn post_dispatch_info_2_xbi_checkout(
        post_dispatch_info: PostDispatchInfo,
        notification_delivery_timeout: T::BlockNumber,
        resolution_status: XBICheckOutStatus,
        actual_delivery_cost: Value,
    ) -> Result<XBICheckOut, Error<T>> {
        let actual_execution_cost =
            if let Some(some_actual_weight) = post_dispatch_info.actual_weight {
                some_actual_weight.into()
            } else {
                0
            };

        let actual_aggregated_cost = actual_delivery_cost + actual_execution_cost;
        Ok(XBICheckOut::new::<T>(
            notification_delivery_timeout,
            post_dispatch_info.encode(),
            resolution_status,
            actual_execution_cost,
            actual_delivery_cost,
            actual_aggregated_cost,
        ))
    }

    fn xbi_checkout_2_post_dispatch_info(
        _xbi_checkout: XBICheckOut,
    ) -> Result<PostDispatchInfo, Error<T>> {
        todo!()
    }
}
