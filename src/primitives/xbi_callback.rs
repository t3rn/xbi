use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_system::pallet_prelude::OriginFor;

use sp_core::{H160, H256, U256};
use sp_std::{marker::PhantomData, vec::Vec};

pub trait XBICallback<T: frame_system::Config + crate::pallet::Config> {
    fn callback(
        xbi_checkin: crate::xbi_format::XBICheckIn<T::BlockNumber>,
        xbi_checkout: crate::xbi_format::XBICheckOut,
    ) -> DispatchResultWithPostInfo;
}

pub struct XBICallbackMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config> XBICallback<T> for XBICallbackMock<T> {
    fn callback(
        xbi_checkin: crate::xbi_format::XBICheckIn<T::BlockNumber>,
        xbi_checkout: crate::xbi_format::XBICheckOut,
    ) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
}

pub struct XBICallbackNoop<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config> XBICallback<T> for XBICallbackNoop<T> {
    fn callback(
        xbi_checkin: crate::xbi_format::XBICheckIn<T::BlockNumber>,
        xbi_checkout: crate::xbi_format::XBICheckOut,
    ) -> DispatchResultWithPostInfo {
        Err(crate::Error::<T>::NoXBICallbackSupported.into())
    }
}
