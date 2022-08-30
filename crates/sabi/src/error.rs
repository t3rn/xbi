use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, ModuleError};
use sp_std::fmt::Debug;

/// A wrapper providing nice access to the module INDEX from the runtime. This allows us to generate
/// error functionality for us in DispatchError.
#[derive(Clone)]
pub struct ModuleErrorProvider<const IDX: u8>(pub Error);

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub enum Error {
    FailedToCastBetweenTypesAddresses,
    FailedToCastBetweenTypesValue,
}

impl<const IDX: u8> Into<DispatchError> for ModuleErrorProvider<IDX> {
    fn into(self) -> DispatchError {
        DispatchError::Module(self.into())
    }
}

impl Into<[u8; 4]> for Error {
    fn into(self) -> [u8; 4] {
        match self {
            Error::FailedToCastBetweenTypesAddresses => [1_u8, 0_u8, 0_u8, 0_u8],
            Error::FailedToCastBetweenTypesValue => [2_u8, 0_u8, 0_u8, 0_u8],
        }
    }
}

impl Into<&'static str> for Error {
    fn into(self) -> &'static str {
        match self {
            Error::FailedToCastBetweenTypesAddresses => "FailedToCastBetweenTypesAddresses",
            Error::FailedToCastBetweenTypesValue => "FailedToCastBetweenTypesValue",
        }
    }
}

impl<const IDX: u8> Into<ModuleError> for ModuleErrorProvider<IDX> {
    fn into(self) -> ModuleError {
        let error: [u8; 4] = self.0.clone().into();
        let msg: &'static str = self.0.into();
        ModuleError {
            index: IDX,
            error,
            message: Some(msg),
        }
    }
}
