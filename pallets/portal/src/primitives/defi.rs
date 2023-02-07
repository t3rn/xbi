use crate::{xbi_abi::AssetId, BalanceOf};
use frame_support::dispatch::DispatchResultWithPostInfo;

use frame_system::pallet_prelude::OriginFor;
use sp_std::marker::PhantomData;

pub trait DeFi<T: frame_system::Config + crate::pallet::Config> {
    fn add_liquidity(
        origin: OriginFor<T>,
        asset_a: AssetId,
        asset_b: AssetId,
        amount_a: BalanceOf<T>,
        amount_b_max_limit: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo;

    fn remove_liquidity(
        origin: OriginFor<T>,
        asset_a: AssetId,
        asset_b: AssetId,
        liquidity_amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo;

    fn swap(
        origin: OriginFor<T>,
        asset_out: AssetId,
        asset_in: AssetId,
        amount: BalanceOf<T>,
        max_limit: BalanceOf<T>,
        discount: bool,
    ) -> DispatchResultWithPostInfo;

    fn get_price(
        origin: OriginFor<T>,
        asset_a: AssetId,
        asset_b: AssetId,
        amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo;
}

pub struct DeFiMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config> DeFi<T> for DeFiMock<T> {
    fn add_liquidity(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount_a: BalanceOf<T>,
        _amount_b_max_limit: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    fn remove_liquidity(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _liquidity_amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    fn swap(
        _origin: OriginFor<T>,
        _asset_out: AssetId,
        _asset_in: AssetId,
        _amount: BalanceOf<T>,
        _max_limit: BalanceOf<T>,
        _discount: bool,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }

    fn get_price(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
}

pub struct DeFiNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config> DeFi<T> for () {
    fn add_liquidity(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount_a: BalanceOf<T>,
        _amount_b_max_limit: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::DefiUnsupported.into())
    }

    fn remove_liquidity(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _liquidity_amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::DefiUnsupported.into())
    }

    fn swap(
        _origin: OriginFor<T>,
        _asset_out: AssetId,
        _asset_in: AssetId,
        _amount: BalanceOf<T>,
        _max_limit: BalanceOf<T>,
        _discount: bool,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::DefiUnsupported.into())
    }

    fn get_price(
        _origin: OriginFor<T>,
        _asset_a: AssetId,
        _asset_b: AssetId,
        _amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::DefiUnsupported.into())
    }
}
