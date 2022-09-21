use crate::error::Error;
use crate::evm::CallEvm;
use crate::wasm::CallWasm;
use substrate_abi::TryConvert;

pub mod error;
pub mod evm;
pub mod wasm;

pub trait SubstrateContractsAbi:
    TryConvert<CallEvm, Outcome = Result<CallWasm, Error>>
    + TryConvert<CallWasm, Outcome = Result<CallEvm, Error>>
{
}

pub struct SubstrateContractAbiConverter;

impl TryConvert<CallEvm> for SubstrateContractAbiConverter {
    type Outcome = Result<CallWasm, Error>;

    fn try_convert(value: CallEvm) -> Self::Outcome {
        CallWasm::try_from(value).map_err(Into::into)
    }
}
impl TryConvert<CallWasm> for SubstrateContractAbiConverter {
    type Outcome = Result<CallEvm, Error>;

    fn try_convert(value: CallWasm) -> Self::Outcome {
        CallEvm::try_from(value).map_err(Into::into)
    }
}

impl SubstrateContractsAbi for SubstrateContractAbiConverter {}
