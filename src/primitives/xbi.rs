use sp_std::marker::PhantomData;

use crate::{xbi_format::*, Error};

pub trait XBIPortal<T: frame_system::Config> {
    fn enter(xbi: XBIFormat, dest: Box<xcm::VersionedMultiLocation>) -> Result<(), Error<T>>;
}

pub struct XBIPortalMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config> XBIPortal<T> for XBIPortalMock<T> {
    fn enter(_xbi: XBIFormat, _dest: Box<xcm::VersionedMultiLocation>) -> Result<(), Error<T>> {
        Ok(())
    }
}
