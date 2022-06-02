use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_system::pallet_prelude::OriginFor;

use sp_core::{H160, H256, U256};
use sp_std::{marker::PhantomData, vec::Vec};

pub trait WASM<T: frame_system::Config + crate::pallet::Config> {
    fn call(
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
    ) -> DispatchResultWithPostInfo;
}

pub struct WASMMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config> WASM<T> for WASMMock<T> {
    fn call(
        _origin: OriginFor<T>,
        _source: H160,
        _target: H160,
        _input: Vec<u8>,
        _value: U256,
        _gas_limit: u64,
        _max_fee_per_gas: U256,
        _max_priority_fee_per_gas: Option<U256>,
        _nonce: Option<U256>,
        _access_list: Vec<(H160, Vec<H256>)>,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
}

pub struct WASMNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config> WASM<T> for WASMNoop<T> {
    fn call(
        _origin: OriginFor<T>,
        _source: H160,
        _target: H160,
        _input: Vec<u8>,
        _value: U256,
        _gas_limit: u64,
        _max_fee_per_gas: U256,
        _max_priority_fee_per_gas: Option<U256>,
        _nonce: Option<U256>,
        _access_list: Vec<(H160, Vec<H256>)>,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::NoWASMSupportedAtDest.into())
    }
}
