use frame_support::dispatch::DispatchResult;
use sp_runtime::traits::StaticLookup;

use frame_system::pallet_prelude::OriginFor;

use sp_std::marker::PhantomData;

pub trait Transfers<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> {
    fn transfer(
        origin: OriginFor<T>,
        target: <T::Lookup as StaticLookup>::Source,
        amount: T::Balance,
    ) -> DispatchResult;
}

pub struct TransfersMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> Transfers<T>
    for TransfersMock<T>
{
    fn transfer(
        _origin: OriginFor<T>,
        _target: <T::Lookup as StaticLookup>::Source,
        _amount: T::Balance,
    ) -> DispatchResult {
        Ok(().into())
    }
}

pub struct TransfersNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> Transfers<T>
    for TransfersNoop<T>
{
    fn transfer(
        _origin: OriginFor<T>,
        _target: <T::Lookup as StaticLookup>::Source,
        _amount: T::Balance,
    ) -> DispatchResult {
        Err(crate::Error::<T>::NoTransfersSupportedAtDest.into())
    }
}
