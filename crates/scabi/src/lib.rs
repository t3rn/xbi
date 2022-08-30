use crate::error::Error;
use crate::evm::CallEvm;
use crate::wasm::CallWasm;
use sp_runtime::traits::TryMorph;

pub mod error;
pub mod evm;
pub mod wasm;

pub trait SubstrateContractsAbi:
    TryMorph<CallEvm, Outcome = Result<CallWasm, Error>>
    + TryMorph<CallWasm, Outcome = Result<CallEvm, Error>>
{
}

pub struct SubstrateContractAbiConverter;

impl TryMorph<CallEvm> for SubstrateContractAbiConverter {
    type Outcome = Result<CallWasm, Error>;

    fn try_morph(value: CallEvm) -> Result<Self::Outcome, ()> {
        Ok(CallWasm::try_from(value).map_err(Into::into))
    }
}
impl TryMorph<CallWasm> for SubstrateContractAbiConverter {
    type Outcome = Result<CallEvm, Error>;

    fn try_morph(value: CallWasm) -> Result<Self::Outcome, ()> {
        Ok(CallEvm::try_from(value).map_err(Into::into))
    }
}

impl SubstrateContractsAbi for SubstrateContractAbiConverter {}
