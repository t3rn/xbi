use frame_support::dispatch::DispatchResult;

use sp_std::marker::PhantomData;

pub trait ORML<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> {
    fn transfer(
        currency_id: u64,
        from: &T::AccountId,
        to: &T::AccountId,
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
        _currency_id: u64,
        _from: &T::AccountId,
        _to: &T::AccountId,
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
        _currency_id: u64,
        _from: &T::AccountId,
        _to: &T::AccountId,
        _amount: T::Balance,
    ) -> DispatchResult {
        Err(crate::Error::<T>::NoTransferORMLSupportedAtDest.into())
    }
}
