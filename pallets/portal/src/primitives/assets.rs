use crate::{xbi_abi::AssetId, BalanceOf};
use frame_support::dispatch::DispatchResult;
use frame_system::pallet_prelude::OriginFor;
use sp_runtime::traits::StaticLookup;
use sp_std::marker::PhantomData;

pub trait Assets<T: frame_system::Config + crate::pallet::Config> {
    fn transfer(
        origin: OriginFor<T>,
        id: AssetId,
        target: <T::Lookup as StaticLookup>::Source,
        amount: BalanceOf<T>,
    ) -> DispatchResult;
}

pub struct AssetsMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config> Assets<T> for AssetsMock<T> {
    fn transfer(
        _origin: OriginFor<T>,
        _id: AssetId,
        _target: <T::Lookup as StaticLookup>::Source,
        _amount: BalanceOf<T>,
    ) -> DispatchResult {
        Ok(())
    }
}

impl<T: frame_system::Config + crate::pallet::Config> Assets<T> for () {
    fn transfer(
        _origin: OriginFor<T>,
        _id: AssetId,
        _target: <T::Lookup as StaticLookup>::Source,
        _amount: BalanceOf<T>,
    ) -> DispatchResult {
        Err(crate::Error::<T>::NoTransferAssetsSupportedAtDest.into())
    }
}
