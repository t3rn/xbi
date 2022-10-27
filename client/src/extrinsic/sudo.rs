use super::*;
use codec::Encode;

pub const SUDO_MODULE: &str = "Sudo";
pub const SUDO_CALL: &str = "sudo";

pub fn wrap_sudo<P, Client, Params, Call>(
    api: Api<P, Client, Params>,
    call: Call,
) -> UncheckedExtrinsicV4<([u8; 2], Call), <Params as ExtrinsicParams>::SignedExtra>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
    MultiSigner: From<P::Public>,
    Client: RpcClient,
    Params: ExtrinsicParams,
    Call: Encode + Clone,
{
    compose_extrinsic!(api, SUDO_MODULE, SUDO_CALL, call)
}
