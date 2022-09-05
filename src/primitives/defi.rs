use frame_support::dispatch::DispatchResultWithPostInfo;


use frame_system::pallet_prelude::OriginFor;

use crate::xbi_abi::AssetId;
use sp_std::marker::PhantomData;

pub trait DeFi<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> {
    fn add_liquidity(
        origin: OriginFor<T>,
        asset_a: AssetId,
        asset_b: AssetId,
        amount_a: T::Balance,
        amount_b_max_limit: T::Balance,
    ) -> DispatchResultWithPostInfo;

    fn remove_liquidity(
        origin: OriginFor<T>,
        asset_a: AssetId,
        asset_b: AssetId,
        liquidity_amount: T::Balance,
    ) -> DispatchResultWithPostInfo;

    fn swap(
        origin: OriginFor<T>,
        asset_out: AssetId,
        asset_in: AssetId,
        amount: T::Balance,
        max_limit: T::Balance,
        discount: bool,
    ) -> DispatchResultWithPostInfo;

    fn get_price(
        origin: OriginFor<T>,
        asset_a: AssetId,
        asset_b: AssetId,
        amount: T::Balance,
    ) -> DispatchResultWithPostInfo;
}

pub struct DeFiMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> DeFi<T>
    for DeFiMock<T>
{
    fn add_liquidity(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount_a: T::Balance,
        _amount_b_max_limit: T::Balance,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    fn remove_liquidity(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _liquidity_amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    fn swap(
        _origin: OriginFor<T>,
        _asset_out: AssetId,
        _asset_in: AssetId,
        _amount: T::Balance,
        _max_limit: T::Balance,
        _discount: bool,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    fn get_price(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
}

pub struct DeFiNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> DeFi<T>
    for DeFiNoop<T>
{
    fn add_liquidity(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount_a: T::Balance,
        _amount_b_max_limit: T::Balance,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::NoDeFiSupportedAtDest.into())
    }

    fn remove_liquidity(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _liquidity_amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::NoDeFiSupportedAtDest.into())
    }

    fn swap(
        _origin: OriginFor<T>,
        _asset_out: AssetId,
        _asset_in: AssetId,
        _amount: T::Balance,
        _max_limit: T::Balance,
        _discount: bool,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::NoDeFiSupportedAtDest.into())
    }

    fn get_price(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount: T::Balance,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::NoDeFiSupportedAtDest.into())
    }
}
