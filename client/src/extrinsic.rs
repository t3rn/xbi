use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};
use substrate_api_client::{
    compose_extrinsic, Api, ExtrinsicParams, RpcClient, UncheckedExtrinsicV4,
};

/// Allow us to catch a panickable event and return an option from it.
/// Since substrate_api_client is largely pretty unsafe, we should ensure the macros are caught appropriately.
#[macro_export]
macro_rules! catch_panicable {
    ($tt:expr) => {{
        use std::panic::catch_unwind;
        catch_unwind(|| $tt).ok()
    }};
}

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
        compose_extrinsic!(api, SUDO_MODULE, SUDO_CALL, call)
    }
}

pub mod xcm {
    use super::*;
    use ::xcm::latest::{
        AssetId, Instruction, Junction, Junctions, MultiAsset, MultiAssetFilter, MultiAssets,
        MultiLocation, OriginKind, WeightLimit, WildMultiAsset, Xcm,
    };
    use ::xcm::prelude::Fungible;
    use ::xcm::{DoubleEncoded, VersionedMultiLocation, VersionedXcm};
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

    pub struct XcmBuilder {
        inner: Xcm<Vec<u8>>,
    }

    impl Default for XcmBuilder {
        fn default() -> Self {
            Self { inner: Xcm::new() }
        }
    }
    impl XcmBuilder {
        pub fn get_relaychain_dest() -> VersionedMultiLocation {
            VersionedMultiLocation::V1(MultiLocation {
                parents: 1,
                interior: Junctions::Here,
            })
        }

        pub fn with_withdraw_asset(mut self, concrete_parent: Option<u8>, amt: u128) -> XcmBuilder {
            self.inner
                .0
                .push(Instruction::WithdrawAsset(MultiAssets::from(vec![
                    MultiAsset {
                        id: AssetId::Concrete(MultiLocation {
                            parents: concrete_parent.unwrap_or(0),
                            interior: Junctions::Here,
                        }),
                        fun: Fungible(amt),
                    },
                ])));
            self
        }

        pub fn with_buy_execution(mut self, concrete_parent: Option<u8>, amt: u128) -> XcmBuilder {
            self.inner.0.push(Instruction::BuyExecution {
                fees: MultiAsset {
                    id: AssetId::Concrete(MultiLocation {
                        parents: concrete_parent.unwrap_or(0),
                        interior: Junctions::Here,
                    }),
                    fun: Fungible(amt),
                },
                weight_limit: WeightLimit::Unlimited,
            });
            self
        }

        pub fn with_transact(mut self, max_weight: Option<u64>, call_hex: Vec<u8>) -> XcmBuilder {
            let call: DoubleEncoded<Vec<u8>> = <DoubleEncoded<_> as From<Vec<u8>>>::from(call_hex);
            self.inner.0.push(Instruction::Transact {
                origin_type: OriginKind::Native,
                require_weight_at_most: max_weight.unwrap_or(1000000000),
                call,
            });
            self
        }

        pub fn with_deposit_asset(
            mut self,
            from_parent: Option<u8>,
            max_assets: u32,
            parachain: u32,
        ) -> XcmBuilder {
            self.inner.0.push(Instruction::DepositAsset {
                assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                max_assets,
                beneficiary: MultiLocation {
                    parents: from_parent.unwrap_or(0),
                    interior: Junctions::X1(Junction::Parachain(parachain)),
                },
            });
            self
        }

        pub fn with_refund_surplus(mut self) -> XcmBuilder {
            self.inner.0.push(Instruction::RefundSurplus);
            self
        }

        pub fn build(self) -> Xcm<Vec<u8>> {
            self.inner
        }
    }
}

pub mod hrmp {

    use codec::Encode;

    pub fn init_open_channel_req(
        parachain_id: u32,
        proposed_max_capacity: Option<u32>,
        proposed_max_message_size: Option<u32>,
    ) -> Vec<u8> {
        // TODO: get index from relaychain
        [
            [23, 0].to_vec(), // call_index
            parachain_id.encode(),
            proposed_max_capacity.unwrap_or(8).encode(),
            proposed_max_message_size.unwrap_or(1024).encode(),
        ]
        .concat()
    }
    pub fn accept_channel_req(parachain_id: u32) -> Vec<u8> {
        // TODO: get index from relaychain
        [
            [23, 1].to_vec(), // call_index
            parachain_id.encode(),
        ]
        .concat()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_init() {
            let expected = hex::decode("1700d10700000800000000a00f00").unwrap();
            assert_eq!(init_open_channel_req(2001, None, Some(1024000)), expected)
        }
        #[test]
        fn test_accept() {
            let expected = hex::decode("1701b80b0000").unwrap();
            assert_eq!(accept_channel_req(3000), expected)
        }
    }
}
