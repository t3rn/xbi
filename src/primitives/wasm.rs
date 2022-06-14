use frame_support::{dispatch::DispatchResultWithPostInfo, weights::Weight};

use sp_std::{marker::PhantomData, vec::Vec};

pub trait WASM<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> {
    fn bare_call(
        origin: T::AccountId,
        dest: T::AccountId,
        value: T::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<T::Balance>,
        data: Vec<u8>,
        debug: bool,
    ) -> DispatchResultWithPostInfo;
}

pub struct WASMMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> WASM<T>
    for WASMMock<T>
{
    fn bare_call(
        _origin: T::AccountId,
        _dest: T::AccountId,
        _value: T::Balance,
        _gas_limit: Weight,
        _storage_deposit_limit: Option<T::Balance>,
        _data: Vec<u8>,
        _debug: bool,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
}

pub struct WASMNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> WASM<T>
    for WASMNoop<T>
{
    fn bare_call(
        _origin: T::AccountId,
        _dest: T::AccountId,
        _value: T::Balance,
        _gas_limit: Weight,
        _storage_deposit_limit: Option<T::Balance>,
        _data: Vec<u8>,
        _debug: bool,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::NoWASMSupportedAtDest.into())
    }
}
