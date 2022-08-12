use frame_support::dispatch::DispatchResult;
use sp_runtime::traits::StaticLookup;

use frame_system::pallet_prelude::OriginFor;

use sp_std::marker::PhantomData;

pub trait Assets<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> {
    fn transfer(
        origin: OriginFor<T>,
        id: u64,
        target: <T::Lookup as StaticLookup>::Source,
        amount: T::Balance,
    ) -> DispatchResult;
}

pub struct AssetsMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> Assets<T>
    for AssetsMock<T>
{
    fn transfer(
        _origin: OriginFor<T>,
        _id: u64,
        _target: <T::Lookup as StaticLookup>::Source,
        _amount: T::Balance,
    ) -> DispatchResult {
        Ok(())
    }
}

pub struct AssetsNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> Assets<T>
    for AssetsNoop<T>
{
    fn transfer(
        _origin: OriginFor<T>,
        _id: u64,
        _target: <T::Lookup as StaticLookup>::Source,
        _amount: T::Balance,
    ) -> DispatchResult {
        Err(crate::Error::<T>::NoTransferAssetsSupportedAtDest.into())
    }
}
