use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use sp_runtime::ModuleError;
use sp_std::fmt::Debug;

/// The number of bytes of the module-specific `error` field defined in [`ModuleError`].
/// In FRAME, this is the maximum encoded size of a pallet error type.
pub const MAX_MODULE_ERROR_ENCODED_SIZE: usize = 4;

/// A wrapper providing access to the module INDEX from the runtime. This allows us to generate
/// error functionality for use in DispatchError.
#[derive(Clone)]
pub struct ModuleErrorProvider<const IDX: u8>(pub Error);

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub enum Error {
    SubstrateAbi(substrate_abi::error::Error),
}

impl From<substrate_abi::error::Error> for Error {
    fn from(e: substrate_abi::error::Error) -> Self {
        Error::SubstrateAbi(e)
    }
}

#[allow(clippy::from_over_into)]
impl<const IDX: u8> Into<DispatchError> for ModuleErrorProvider<IDX> {
    fn into(self) -> DispatchError {
        DispatchError::Module(self.into())
    }
}

#[allow(clippy::from_over_into)]
impl<const IDX: u8> Into<[u8; MAX_MODULE_ERROR_ENCODED_SIZE]> for ModuleErrorProvider<IDX> {
    fn into(self) -> [u8; MAX_MODULE_ERROR_ENCODED_SIZE] {
        match self.0 {
            Error::SubstrateAbi(e) => {
                let inner: ModuleError = substrate_abi::error::ModuleErrorProvider::<IDX>(e).into();
                [1u8, inner.error[0], 0_u8, 0_u8]
            }
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<&'static str> for Error {
    fn into(self) -> &'static str {
        match self {
            Error::SubstrateAbi(s) => match s {
                substrate_abi::error::Error::FailedToCastBetweenTypesAddresses => {
                    concat!("SubstrateAbi", "FailedToCastBetweenTypesAddresses")
                }
                substrate_abi::error::Error::FailedToCastBetweenTypesValue => {
                    concat!("SubstrateAbi", "FailedToCastBetweenTypesValue")
                }
                substrate_abi::error::Error::FailedToAssociateTypes => {
                    concat!("SubstrateAbi", "FailedToAssociateTypes")
                }
            },
        }
    }
}

#[allow(clippy::from_over_into)]
impl<const IDX: u8> Into<ModuleError> for ModuleErrorProvider<IDX> {
    fn into(self) -> ModuleError {
        let error: [u8; MAX_MODULE_ERROR_ENCODED_SIZE] = self.clone().into();
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
        let m: ModuleError = ModuleErrorProvider::<1>(Error::SubstrateAbi(
            substrate_abi::error::Error::FailedToCastBetweenTypesValue,
        ))
        .into();
        assert_eq!(m.index, 1);
        // 1 is the index of the error variant for the scabi substrate variant, so error[0] is 1
        // 0 is none, so we count from normal for these iterations
        // 2 is the index of the error variant for the substrate abi, so we map error[1] to 2.
        assert_eq!(m.error, [1_u8, 2_u8, 0_u8, 0_u8]);
    }

    #[test]
    fn test_into_str() {
        let e: &str =
            Error::SubstrateAbi(substrate_abi::error::Error::FailedToCastBetweenTypesAddresses)
                .into();
        assert_eq!(e, "SubstrateAbiFailedToCastBetweenTypesAddresses");
        let e: &str =
            Error::SubstrateAbi(substrate_abi::error::Error::FailedToCastBetweenTypesValue).into();
        assert_eq!(e, "SubstrateAbiFailedToCastBetweenTypesValue");
        let e: &str =
            Error::SubstrateAbi(substrate_abi::error::Error::FailedToAssociateTypes).into();
        assert_eq!(e, "SubstrateAbiFailedToAssociateTypes");
    }
}
