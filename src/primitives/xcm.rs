use sp_std::marker::PhantomData;


use xcm::latest::{prelude::*, MultiLocation};

pub trait XCM<T: frame_system::Config> {
    fn send_xcm(
        interior: impl Into<Junctions>,
        dest: impl Into<MultiLocation>,
        message: Xcm<()>,
    ) -> Result<(), SendError>;
}

pub struct XCMMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config> XCM<T> for XCMMock<T> {
    fn send_xcm(
        _interior: impl Into<Junctions>,
        _dest: impl Into<MultiLocation>,
        _message: Xcm<()>,
    ) -> Result<(), SendError> {
        Ok(())
    }
}
