use sp_std::marker::PhantomData;

pub trait XBICallback<T: frame_system::Config + crate::pallet::Config> {
    fn callback(
        xbi_checkin: xp_format::XbiCheckIn<T::BlockNumber>,
        xbi_checkout: xp_format::XbiCheckOut,
    );
}

pub struct XBICallbackMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config + crate::pallet::Config> XBICallback<T> for XBICallbackMock<T> {
    fn callback(
        _xbi_checkin: xp_format::XbiCheckIn<T::BlockNumber>,
        _xbi_checkout: xp_format::XbiCheckOut,
    ) {
    }
}

impl<T: frame_system::Config + crate::pallet::Config> XBICallback<T> for () {
    fn callback(
        _xbi_checkin: xp_format::XbiCheckIn<T::BlockNumber>,
        _xbi_checkout: xp_format::XbiCheckOut,
    ) {
    }
}
