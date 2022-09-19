use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

use crate::{xbi_format::*, Error};

pub trait XBIPortal<T: frame_system::Config> {
    fn do_check_in_xbi(xbi: XBIFormat) -> Result<(), Error<T>>;
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct XBIPortalMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config> XBIPortal<T> for XBIPortalMock<T> {
    fn do_check_in_xbi(_xbi: XBIFormat) -> Result<(), Error<T>> {
        Ok(())
    }
}
