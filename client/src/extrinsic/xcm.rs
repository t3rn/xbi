use super::*;
use ::xcm::prelude::*;
use substrate_api_client::compose_call;

pub const XCM_MODULE: &str = "PolkadotXcm";
pub const XCM_SEND: &str = "send";

pub fn xcm_compose<P, Client, Params>(
    api: Api<P, Client, Params>,
    dest: VersionedMultiLocation,
    msg: VersionedXcm<Vec<u8>>,
) -> ([u8; 2], VersionedMultiLocation, VersionedXcm<Vec<u8>>)
where
    P: Pair,
    MultiSignature: From<P::Signature>,
    MultiSigner: From<P::Public>,
    Client: RpcClient,
    Params: ExtrinsicParams,
{
    compose_call!(api.metadata, XCM_MODULE, XCM_SEND, dest, msg)
}

#[allow(clippy::type_complexity)] // It's simply complex
pub fn xcm_send<P, Client, Params>(
    api: Api<P, Client, Params>,
    dest: VersionedMultiLocation,
    msg: VersionedXcm<Vec<u8>>,
) -> UncheckedExtrinsicV4<
    ([u8; 2], VersionedMultiLocation, VersionedXcm<Vec<u8>>),
    <Params as ExtrinsicParams>::SignedExtra,
>
where
    P: Pair,
    MultiSignature: From<P::Signature>,
    MultiSigner: From<P::Public>,
    Client: RpcClient,
    Params: ExtrinsicParams,
{
    compose_extrinsic!(api, XCM_MODULE, XCM_SEND, dest, msg)
}
