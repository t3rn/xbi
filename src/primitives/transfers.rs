use frame_support::dispatch::DispatchError;

use sp_std::marker::PhantomData;

pub trait Transfers<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> {
    fn transfer(
        source: &T::AccountId,
        dest: &T::AccountId,
        amount: T::Balance,
        keep_alive: bool,
    ) -> Result<T::Balance, DispatchError>;
}

pub struct TransfersMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> Transfers<T>
    for TransfersMock<T>
{
    fn transfer(
        _source: &T::AccountId,
        _dest: &T::AccountId,
        amount: T::Balance,
        _keep_alive: bool,
    ) -> Result<T::Balance, DispatchError> {
        Ok(amount)
    }
}

pub struct TransfersNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config + pallet_balances::Config> Transfers<T>
    for TransfersNoop<T>
{
    fn transfer(
        _source: &T::AccountId,
        _dest: &T::AccountId,
        _amount: T::Balance,
        _keep_alive: bool,
    ) -> Result<T::Balance, DispatchError> {
        Err(crate::Error::<T>::NoTransferSupportedAtDest.into())
    }
}
