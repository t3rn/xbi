use frame_support::dispatch::DispatchResult;
use sp_runtime::traits::StaticLookup;

use frame_system::pallet_prelude::OriginFor;

use sp_std::marker::PhantomData;

pub trait ORML<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> {
    fn transfer(
        origin: OriginFor<T>,
        id: u32,
        target: <T::Lookup as StaticLookup>::Source,
        amount: T::Balance,
    ) -> DispatchResult;
}

pub struct ORMLMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> ORML<T>
    for ORMLMock<T>
{
    fn transfer(
        _origin: OriginFor<T>,
        _id: u32,
        _target: <T::Lookup as StaticLookup>::Source,
        _amount: T::Balance,
    ) -> DispatchResult {
        Ok(().into())
    }
}

pub struct ORMLNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> ORML<T>
    for ORMLNoop<T>
{
    fn transfer(
        _origin: OriginFor<T>,
        _id: u32,
        _target: <T::Lookup as StaticLookup>::Source,
        _amount: T::Balance,
    ) -> DispatchResult {
        Err(crate::Error::<T>::NoTransferORMLSupportedAtDest.into())
    }
}
