use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, ModuleError};
use sp_std::fmt::Debug;

/// The number of bytes of the module-specific `error` field defined in [`ModuleError`].
/// In FRAME, this is the maximum encoded size of a pallet error type.
pub const MAX_MODULE_ERROR_ENCODED_SIZE: usize = 4;

/// A wrapper providing access to the module INDEX from the runtime. This allows us to generate
/// error functionality for use in DispatchError.
#[derive(Clone)]
pub struct ModuleErrorProvider<const IDX: u8>(pub Error);

// TODO: needs improving on names
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub enum Error {
    FailedToCastBetweenTypesAddresses,
    FailedToCastBetweenTypesValue,
    FailedToAssociateTypes,
}

#[allow(clippy::from_over_into)]
impl<const IDX: u8> Into<DispatchError> for ModuleErrorProvider<IDX> {
    fn into(self) -> DispatchError {
        DispatchError::Module(self.into())
    }
}

#[allow(clippy::from_over_into)]
impl Into<[u8; MAX_MODULE_ERROR_ENCODED_SIZE]> for Error {
    fn into(self) -> [u8; MAX_MODULE_ERROR_ENCODED_SIZE] {
        match self {
            Error::FailedToCastBetweenTypesAddresses => [1_u8, 0_u8, 0_u8, 0_u8],
            Error::FailedToCastBetweenTypesValue => [2_u8, 0_u8, 0_u8, 0_u8],
            Error::FailedToAssociateTypes => [3_u8, 0_u8, 0_u8, 0_u8],
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<&'static str> for Error {
    fn into(self) -> &'static str {
        match self {
            Error::FailedToCastBetweenTypesAddresses => "FailedToCastBetweenTypesAddresses",
            Error::FailedToCastBetweenTypesValue => "FailedToCastBetweenTypesValue",
            Error::FailedToAssociateTypes => "FailedToAssociateTypes",
        }
    }
}

#[allow(clippy::from_over_into)]
impl<const IDX: u8> Into<ModuleError> for ModuleErrorProvider<IDX> {
    fn into(self) -> ModuleError {
        let error: [u8; MAX_MODULE_ERROR_ENCODED_SIZE] = self.0.clone().into();
        let msg: &'static str = self.0.into();
        ModuleError {
            index: IDX,
            error,
            message: Some(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_error_provides_correct_byte_sequence() {
        let m: ModuleError = ModuleErrorProvider::<1>(Error::FailedToCastBetweenTypesValue).into();
        assert_eq!(m.index, 1);
        assert_eq!(m.error, [2_u8, 0_u8, 0_u8, 0_u8]);
    }
}
