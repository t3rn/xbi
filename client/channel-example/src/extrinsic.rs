use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};
use substrate_api_client::{
    compose_extrinsic, Api, ExtrinsicParams, RpcClient, UncheckedExtrinsicV4,
};

pub mod sudo {
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
        compose_extrinsic!(api, "Sudo", "sudo", call)
    }
}
pub mod xcm {
    use super::*;
    use ::xcm::{VersionedMultiLocation, VersionedXcm};
    use codec::Encode;
    use substrate_api_client::compose_call;

    pub const XCM_MODULE: &str = "PolkadotXcm";
    pub const XCM_SEND: &str = "send";

    pub fn xcm_send<P, Client, Params>(
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn asdas() {}
    }
}
pub mod hrmp {
    // TODO: build encoded call
}
