use sp_core::Pair;
use sp_runtime::{MultiSignature, MultiSigner};
use substrate_api_client::{
    compose_extrinsic, Api, ExtrinsicParams, Metadata, RpcClient, UncheckedExtrinsicV4,
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
    use ::xcm::latest::NetworkId;
    use ::xcm::latest::{
        AssetId, Instruction, Junction, Junctions, MultiAsset, MultiAssetFilter, MultiAssets,
        MultiLocation, OriginKind, WeightLimit, Xcm,
    };
    use ::xcm::prelude::All;
    use ::xcm::prelude::Fungible;
    use ::xcm::{DoubleEncoded, VersionedMultiLocation, VersionedXcm};
    use codec::{Codec, Decode, Encode};
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

    pub struct MultiLocationBuilder {
        inner: MultiLocation,
    }

    impl Default for MultiLocationBuilder {
        fn default() -> Self {
            Self {
                inner: Default::default(),
            }
        }
    }

    impl MultiLocationBuilder {
        pub fn get_relaychain_dest() -> VersionedMultiLocation {
            VersionedMultiLocation::V1(MultiLocationBuilder::new_native(Some(1)).build())
        }

        pub fn new_native(parent: Option<u8>) -> Self {
            Self {
                inner: MultiLocation {
                    parents: parent.unwrap_or_default(),
                    interior: Junctions::Here,
                },
            }
        }

        pub fn new_parachain(parent: Option<u8>, parachain: u32) -> Self {
            Self {
                inner: MultiLocation {
                    parents: parent.unwrap_or_default(),
                    interior: Junctions::X1(Junction::Parachain(parachain)),
                },
            }
        }

        pub fn new_account(parent: Option<u8>, account: [u8; 32]) -> Self {
            Self {
                inner: MultiLocation {
                    parents: parent.unwrap_or_default(),
                    interior: Junctions::X1(Junction::AccountId32 {
                        network: NetworkId::Any,
                        id: account,
                    }),
                },
            }
        }

        pub fn with_junction(mut self, jnc: Junction) -> Self {
            match self.inner.interior {
                Junctions::X8(t, u, v, w, x, y, z, _a) => {
                    self.inner.interior = Junctions::X8(t, u, v, w, x, y, z, jnc);
                }
                _ => {
                    self.inner.push_interior(jnc);
                } // Overwrite the last action
            }
            self
        }

        pub fn build(self) -> MultiLocation {
            self.inner
        }
    }
    pub struct XcmBuilder<T> {
        inner: Xcm<T>,
    }

    impl<T> Default for XcmBuilder<T> {
        fn default() -> Self {
            Self { inner: Xcm::new() }
        }
    }

    impl<T: Codec> XcmBuilder<T> {
        pub fn with_transfer_self_reserve(
            mut self,
            assets: MultiAssets,
            fee: u128,
            dest: MultiLocation,
            recipient: MultiLocation,
            weight_limit: Option<u64>,
        ) -> Self {
            let reserve_xcm = XcmBuilder::default()
                .with_buy_execution(dest.clone(), fee, weight_limit.map(WeightLimit::Limited))
                .with_deposit_asset(recipient, assets.len() as u32)
                .build();
            self.inner.0.push(Instruction::TransferReserveAsset {
                assets,
                dest,
                // This is injected and called by the dest (self)
                xcm: reserve_xcm,
            });
            self
        }

        pub fn with_transfer_reserve_to_reserve(
            mut self,
            assets: MultiAssets,
            fee: u128,
            reserve: MultiLocation,
            recipient: MultiLocation,
            weight_limit: Option<u64>,
        ) -> Self {
            let injected_xcm = XcmBuilder::default()
                .with_buy_execution(reserve.clone(), fee, weight_limit.map(WeightLimit::Limited))
                .with_deposit_asset(recipient, assets.len() as u32)
                .build();
            self.inner.0.push(Instruction::InitiateReserveWithdraw {
                assets: MultiAssetFilter::Wild(All),
                reserve,
                // This is injected and called by the reserve(target)
                xcm: injected_xcm,
            });
            self
        }

        pub fn with_transfer(
            mut self,
            assets: MultiAssets,
            execution_fee: u128,
            reserve: MultiLocation,
            dest: MultiLocation,
            recipient: MultiLocation,
            weight_limit: Option<u64>,
            // Whether the reserve can teleport the transfer
            should_teleport: bool,
        ) -> Self {
            let mut reanchored_dest = dest.clone();
            if reserve == MultiLocation::parent() {
                match dest {
                    MultiLocation {
                        parents,
                        interior: Junctions::X1(Junction::Parachain(id)),
                    } if parents == 1 => {
                        reanchored_dest = Junction::Parachain(id).into();
                    }
                    _ => {}
                }
            }

            self.inner
                .0
                .push(Instruction::WithdrawAsset(assets.clone()));

            self.inner.0.push(Instruction::InitiateReserveWithdraw {
                assets: MultiAssetFilter::Wild(All),
                reserve: reserve.clone(),
                xcm: if should_teleport {
                    XcmBuilder::default()
                        .with_buy_execution(
                            reserve,
                            execution_fee / 2,
                            weight_limit.map(WeightLimit::Limited),
                        )
                        .with_initiate_teleport(
                            reanchored_dest,
                            recipient,
                            execution_fee / 2,
                            weight_limit,
                            assets,
                        )
                        .build()
                } else {
                    XcmBuilder::default()
                        .with_buy_execution(
                            reserve,
                            execution_fee / 2,
                            weight_limit.map(WeightLimit::Limited),
                        )
                        .with_deposit_reserve_asset(
                            reanchored_dest,
                            recipient,
                            execution_fee / 2,
                            weight_limit,
                            assets,
                        )
                        .build()
                },
            });
            self
        }

        pub fn with_initiate_teleport(
            mut self,
            dest: MultiLocation,
            recipient: MultiLocation,
            execution_fee: u128,
            weight_limit: Option<u64>,
            assets: MultiAssets,
        ) -> XcmBuilder<T> {
            self.inner.0.push(Instruction::InitiateTeleport {
                assets: MultiAssetFilter::Wild(All),
                dest: dest.clone(),
                xcm: XcmBuilder::default()
                    .with_buy_execution(dest, execution_fee, weight_limit.map(WeightLimit::Limited))
                    .with_deposit_asset(recipient, assets.len() as u32)
                    .build(),
            });
            self
        }

        pub fn with_deposit_reserve_asset(
            mut self,
            dest: MultiLocation,
            recipient: MultiLocation,
            execution_fee: u128,
            weight_limit: Option<u64>,
            assets: MultiAssets,
        ) -> XcmBuilder<T> {
            self.inner.0.push(Instruction::DepositReserveAsset {
                assets: MultiAssetFilter::Wild(All),
                max_assets: assets.len() as u32,
                dest: dest.clone(),
                xcm: XcmBuilder::default()
                    .with_buy_execution(dest, execution_fee, weight_limit.map(WeightLimit::Limited))
                    .with_deposit_asset(recipient, assets.len() as u32)
                    .build(),
            });
            self
        }

        pub fn with_withdraw_asset(mut self, asset: MultiLocation, amt: u128) -> XcmBuilder<T> {
            self.inner
                .0
                .push(Instruction::WithdrawAsset(MultiAssets::from(vec![
                    MultiAsset {
                        id: AssetId::Concrete(asset),
                        fun: Fungible(amt),
                    },
                ])));
            self
        }

        pub fn with_buy_execution(
            mut self,
            asset: MultiLocation,
            amt: u128,
            weight_limit: Option<WeightLimit>,
        ) -> XcmBuilder<T> {
            self.inner.0.push(Instruction::BuyExecution {
                fees: MultiAsset {
                    id: AssetId::Concrete(asset),
                    fun: Fungible(amt),
                },
                weight_limit: weight_limit.unwrap_or(WeightLimit::Unlimited),
            });
            self
        }

        pub fn with_transact(mut self, max_weight: Option<u64>, call_hex: T) -> XcmBuilder<T> {
            let call: DoubleEncoded<T> = Encode::encode(&call_hex).into();
            self.inner.0.push(Instruction::Transact {
                origin_type: OriginKind::Native,
                require_weight_at_most: max_weight.unwrap_or(1_000_000_000),
                call,
            });
            self
        }

        pub fn with_deposit_asset(
            mut self,
            beneficiary: MultiLocation,
            max_assets: u32,
        ) -> XcmBuilder<T> {
            self.inner.0.push(Instruction::DepositAsset {
                assets: MultiAssetFilter::Wild(All),
                max_assets,
                beneficiary,
            });
            self
        }

        pub fn with_refund_surplus(mut self) -> XcmBuilder<T> {
            self.inner.0.push(Instruction::RefundSurplus);
            self
        }

        pub fn build(self) -> Xcm<T> {
            self.inner
        }
    }
}

pub mod hrmp {

    use crate::extrinsic::index_from_metadata;
    use codec::Encode;
    use sp_core::sr25519::Pair;
    use sp_keyring::AccountKeyring;
    use substrate_api_client::rpc::WsRpcClient;
    use substrate_api_client::{Api, Metadata, PlainTipExtrinsicParams};

    const HRMP_PALLET: &str = "Hrmp";
    const HRMP_INIT_CHANNEL: &str = "hrmp_init_open_channel";
    const HRMP_ACCEPT_CHANNEL: &str = "hrmp_accept_open_channel";

    const ROCOCO_RELAYCHAIN_OFFICIAL_HOST: &str = "wss://rococo-rpc.polkadot.io";

    #[memoize::memoize]
    pub fn get_relaychain_metadata(host: Option<String>) -> Metadata {
        let pair = AccountKeyring::Alice.pair();

        let host = host.unwrap_or_else(|| ROCOCO_RELAYCHAIN_OFFICIAL_HOST.to_string());
        let client = WsRpcClient::new(&host);
        let api = Api::<Pair, _, PlainTipExtrinsicParams>::new(client)
            .map(|api| api.set_signer(pair))
            .expect("Failed to initiate the rpc client");

        api.get_metadata()
            .ok()
            .and_then(|meta| meta.try_into().ok())
            .expect("Failed to read metadata from relaychain")
    }

    pub fn init_open_channel_req(
        parachain_id: u32,
        proposed_max_capacity: Option<u32>,
        proposed_max_message_size: Option<u32>,
        relaychain_meta: Option<Metadata>,
    ) -> Vec<u8> {
        let pallet_and_call = if let Some(relaychain_meta) = relaychain_meta {
            let (pallet_idx, call_idx) = index_from_metadata(
                relaychain_meta,
                HRMP_PALLET.to_string(),
                HRMP_INIT_CHANNEL.to_string(),
            )
            .unwrap();
            [pallet_idx, call_idx]
        } else {
            [23, 0] // roco
        }
        .to_vec();
        [
            pallet_and_call,
            parachain_id.encode(),
            proposed_max_capacity.unwrap_or(8).encode(),
            proposed_max_message_size.unwrap_or(1024).encode(),
        ]
        .concat()
    }

    pub fn accept_channel_req(parachain_id: u32, relaychain_meta: Option<Metadata>) -> Vec<u8> {
        let pallet_and_call = if let Some(relaychain_meta) = relaychain_meta {
            let (pallet_idx, call_idx) = index_from_metadata(
                relaychain_meta,
                HRMP_PALLET.to_string(),
                HRMP_ACCEPT_CHANNEL.to_string(),
            )
            .unwrap();
            [pallet_idx, call_idx]
        } else {
            [23, 1] // roco
        }
        .to_vec();
        [
            pallet_and_call, // call_index
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
            assert_eq!(
                init_open_channel_req(2001, None, Some(1024000), None),
                expected
            )
        }
        #[test]
        fn test_accept() {
            let expected = hex::decode("1701b80b0000").unwrap();
            assert_eq!(accept_channel_req(3000, None), expected)
        }
    }
}

pub fn index_from_metadata(metadata: Metadata, pallet: String, call: String) -> Option<(u8, u8)> {
    metadata.pallets.get(&pallet).and_then(|pallet| {
        pallet
            .calls
            .get(&call)
            .map(|call_index| (pallet.index, *call_index))
    })
}
