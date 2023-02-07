use crate::Error;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;
use xbi_format::*;

pub trait XBIPortal<T: frame_system::Config> {
    fn do_check_in_xbi(xbi: XbiFormat) -> Result<(), Error<T>>;
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct XBIPortalMock<T> {
    _phantom: PhantomData<T>,
}

impl<T: frame_system::Config> XBIPortal<T> for XBIPortalMock<T> {
    fn do_check_in_xbi(_xbi: XbiFormat) -> Result<(), Error<T>> {
        Ok(())
    }
}