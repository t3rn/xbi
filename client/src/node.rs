use crate::extrinsic::hrmp::{accept_channel_req, get_relaychain_metadata, init_open_channel_req};
use crate::manager::MessageManager;
use crate::{catch_panicable, Message};
use codec::Encode;
use sp_core::crypto::Pair as PairExt;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::path::PathBuf;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata, PlainTipExtrinsicParams, XtStatus};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use xcm::latest::{Junction, MultiAsset, MultiAssets};
use xcm::prelude::OriginKind;
use xcm::VersionedXcm;
use xcm_primitives::{MultiLocationBuilder, XcmBuilder};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Command {
    #[serde(with = "hex::serde")]
    Sudo(Vec<u8>),
    HrmpInitChannel(u32),
    HrmpAcceptChannel(u32),
    UpdateRelayChain(String),
    TransferReserve {
        reserve_is_self: bool,
        asset: u64,
        amount: u128,
        dest_parachain: u32,
        recipient: String,
    },
    TopupSelfReserve {
        asset: u64,
        amount: u128,
    },
    Teleport {
        asset: u64,
        amount: u128,
        dest_parachain: u32,
    },
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct NodeConfig {
    pub parachain_id: u32,
    pub host: String,
    pub sleep_time_secs: u64,
    pub key_seed: Option<PathBuf>,
}

impl NodeConfig {
    pub fn _new(
        parachain_id: u32,
        host: String,
        sleep_time_secs: Option<u64>,
        key_pair: Option<PathBuf>,
    ) -> Self {
        NodeConfig {
            parachain_id,
            host,
            sleep_time_secs: sleep_time_secs.unwrap_or(5),
            key_seed: key_pair,
        }
    }

    pub fn read_key_or_alice(&self) -> Pair {
        self.key_seed
            .clone()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|contents| Pair::from_string(contents.replace(' ', "/").trim(), None).ok())
            .unwrap_or_else(|| AccountKeyring::Alice.pair())
    }
}

impl MessageManager<Command> for NodeConfig {
    fn start(&self, mut rx: Receiver<Command>, _tx: Sender<Message>) -> anyhow::Result<()> {
        log::info!("Starting node manager for host {}", self.host);

        let host_shadow = self.host.clone();
        let sleep_shadow = self.sleep_time_secs;
        let key_pair_shadow = self.read_key_or_alice();
        let parachain_id_shadow = self.parachain_id;
        let _ = tokio::spawn(async move {
            let client = WsRpcClient::new(&host_shadow);
            let api = Api::<Pair, _, PlainTipExtrinsicParams>::new(client)
                .map(|api| api.set_signer(key_pair_shadow))
                .expect("Failed to initiate the rpc client");

            let meta: Metadata = api
                .get_metadata()
                .ok()
                .and_then(|meta| meta.try_into().ok())
                .expect("{host_shadow} No metadata was returned");

            // TODO: assert capabilities on parachain, ensuring they have X installed
            log::info!("{host_shadow} exposed {} pallets", meta.pallets.len());
            log::info!("{host_shadow} exposed {} events", meta.events.len());
            log::info!("{host_shadow} exposed {} errors", meta.errors.len());

            let mut relaychain_host = None;
            while let Some(msg) = rx.recv().await {
                use Command::*;
                log::trace!("Received request: {:?}", msg);

                let api_shadow = api.clone();
                let host_shadow = host_shadow.clone();

                match msg {
                    Sudo(bytes) => {
                        log::info!("sending sudo call {:?}", bytes);

                        if let Some(extrinsic) = catch_panicable!(
                            crate::extrinsic::sudo::wrap_sudo(api_shadow.clone(), bytes)
                        ) {
                            tokio::task::spawn_blocking(move || {
                                let _ = api_shadow
                                    .send_extrinsic(extrinsic.hex_encode(), XtStatus::InBlock)
                                    .map_err(|err| {
                                        log::error!(
                                            "{host_shadow} failed to send request {:?}",
                                            err
                                        )
                                    });
                            });
                        }
                    }
                    HrmpInitChannel(parachain) => {
                        let relaychain_meta = get_relaychain_metadata(relaychain_host.clone());

                        let call = XcmBuilder::default()
                            .with_withdraw_concrete_asset(
                                MultiLocationBuilder::new_native().build(),
                                1_000_000_000_000,
                            )
                            .with_buy_execution(
                                MultiLocationBuilder::new_native().build(),
                                1_000_000_000_000,
                                None,
                            )
                            .with_transact(
                                Some(OriginKind::SovereignAccount),
                                None,
                                init_open_channel_req(parachain, None, None, Some(relaychain_meta))
                                    .encode(),
                            )
                            .with_refund_surplus()
                            .with_deposit_asset(
                                MultiLocationBuilder::new_parachain(parachain_id_shadow).build(),
                                1,
                            )
                            .build();

                        let call = crate::extrinsic::xcm::xcm_compose(
                            api_shadow.clone(),
                            MultiLocationBuilder::get_relaychain_dest(),
                            VersionedXcm::V2(call),
                        );
                        crate::send_sudo_msg!(api_shadow, call, host_shadow);
                    }
                    HrmpAcceptChannel(parachain) => {
                        let relaychain_meta = get_relaychain_metadata(relaychain_host.clone());

                        let call = XcmBuilder::default()
                            .with_withdraw_concrete_asset(
                                MultiLocationBuilder::new_native().build(),
                                1_000_000_000_000,
                            )
                            .with_buy_execution(
                                MultiLocationBuilder::new_native().build(),
                                1_000_000_000_000,
                                None,
                            )
                            .with_transact(
                                Some(OriginKind::SovereignAccount),
                                None,
                                accept_channel_req(parachain, Some(relaychain_meta)).encode(),
                            )
                            .with_refund_surplus()
                            .with_deposit_asset(
                                MultiLocationBuilder::new_parachain(parachain_id_shadow).build(),
                                1,
                            )
                            .build();

                        let call = crate::extrinsic::xcm::xcm_compose(
                            api_shadow.clone(),
                            MultiLocationBuilder::get_relaychain_dest(),
                            VersionedXcm::V2(call),
                        );
                        crate::send_sudo_msg!(api_shadow.clone(), call, host_shadow);
                    }
                    TransferReserve {
                        reserve_is_self,
                        asset,
                        amount,
                        dest_parachain,
                        recipient: _,
                    } => {
                        let asset = MultiAsset::from((
                            MultiLocationBuilder::new_native()
                                .with_junction(Junction::GeneralIndex(asset as u128)) // TODO: no
                                .build(),
                            amount,
                        ));
                        let call = if reserve_is_self {
                            XcmBuilder::default().with_transfer_self_reserve(
                                MultiAssets::from(vec![asset]),
                                1_000_000_000_000,
                                MultiLocationBuilder::new_parachain(dest_parachain).build(),
                                MultiLocationBuilder::new_parachain(dest_parachain).build(),
                                None,
                            )
                        } else {
                            XcmBuilder::default().with_transfer(
                                MultiAssets::from(vec![asset]),
                                1_000_000_000_000,
                                MultiLocationBuilder::new_parachain(3000)
                                    .with_parents(1)
                                    .build(),
                                MultiLocationBuilder::new_parachain(dest_parachain).build(),
                                MultiLocationBuilder::new_native().build(),
                                None,
                                false,
                            )
                        };

                        let call = crate::extrinsic::xcm::xcm_compose(
                            api_shadow.clone(),
                            MultiLocationBuilder::get_relaychain_dest(),
                            VersionedXcm::V2(call.build()),
                        );
                        crate::send_sudo_msg!(api_shadow.clone(), call, host_shadow);
                    }
                    TopupSelfReserve { amount, .. } => {
                        let asset = MultiAsset::from((
                            MultiLocationBuilder::new_native()
                                // .with_junction(Junction::GeneralIndex(asset as u128)) // TODO: no
                                .build(),
                            amount,
                        ));
                        let _assets = MultiAssets::from(vec![asset]);
                        let _self_location =
                            MultiLocationBuilder::new_parachain(parachain_id_shadow)
                                .with_parents(1)
                                .build();
                        // let call = XcmBuilder::default().with_transfer_reserve_to_reserve(
                        //     assets,
                        //     1_000_000_000_000,
                        //     self_location.clone(),
                        //     self_location.clone(),
                        //     None,
                        // );
                        // let call = XcmBuilder::default()
                        //     .with_withdraw_asset(self_location.clone(), 1_000_000_000_000)
                        //     .with_deposit_reserve_asset(
                        //         self_location.clone(),
                        //         self_location.clone(),
                        //         1_000_000_000_000,
                        //         None,
                        //         assets,
                        //     );
                        let call = XcmBuilder::default().with_withdraw_concrete_asset(
                            MultiLocationBuilder::new_native().build(),
                            1_000_000_000_000,
                        );
                        // .with_deposit_reserve_asset(
                        //     self_location.clone(),
                        //     self_location.clone(),
                        //     1_000_000_000_000,
                        //     None,
                        //     assets,
                        // );

                        let call = crate::extrinsic::xcm::xcm_compose(
                            api_shadow.clone(),
                            MultiLocationBuilder::get_relaychain_dest(),
                            VersionedXcm::V2(call.build()),
                        );
                        crate::send_sudo_msg!(api_shadow.clone(), call, host_shadow);
                    }
                    Teleport { amount, .. } => {
                        let asset = MultiAsset::from((
                            MultiLocationBuilder::new_native()
                                // .with_junction(Junction::GeneralIndex(asset as u128)) // TODO: no
                                .build(),
                            amount,
                        ));
                        let assets = MultiAssets::from(vec![asset]);

                        let call = XcmBuilder::default()
                            .with_withdraw_concrete_asset(
                                MultiLocationBuilder::new_native().with_parents(1).build(),
                                amount,
                            )
                            .with_initiate_teleport(
                                MultiLocationBuilder::new_parachain(3)
                                    .with_parents(1)
                                    .build(),
                                MultiLocationBuilder::new_parachain(3)
                                    .with_parents(1)
                                    .build(),
                                amount / 2,
                                None,
                                assets,
                            );

                        let call = crate::extrinsic::xcm::xcm_send(
                            api_shadow.clone(),
                            MultiLocationBuilder::new_parachain(3)
                                .with_parents(1)
                                .build()
                                .into(),
                            VersionedXcm::V2(call.build()),
                        );
                        crate::send_msg!(api_shadow, call, host_shadow);
                    }
                    UpdateRelayChain(new_host) => {
                        log::info!("{host_shadow} updating relay chain to {new_host}");
                        relaychain_host = Some(new_host);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(sleep_shadow)).await;
            }
        });
        Ok(())
    }
}

#[macro_export]
macro_rules! send_sudo_msg {
    ($api_shadow:expr, $call:tt, $host_shadow:tt) => {{
        let shadowed = $api_shadow.clone();
        if let Some(extrinsic) =
            catch_panicable!($crate::extrinsic::sudo::wrap_sudo(shadowed.clone(), $call))
        {
            tokio::task::spawn_blocking(move || {
                let _ = shadowed
                    .clone()
                    .send_extrinsic(extrinsic.hex_encode(), XtStatus::InBlock)
                    .map_err(|err| log::error!("{} failed to send request {:?}", $host_shadow, err))
                    .map(|ok| log::info!("{} completed request {:?}", $host_shadow, ok));
            });
        }
    }};
}
#[macro_export]
macro_rules! send_msg {
    ($api_shadow:expr, $call:tt, $host_shadow:tt) => {{
        let shadowed = $api_shadow.clone();

        tokio::task::spawn_blocking(move || {
            let _ = shadowed
                .send_extrinsic($call.hex_encode(), XtStatus::InBlock)
                .map_err(|err| log::error!("{} failed to send request {:?}", $host_shadow, err))
                .map(|ok| log::info!("{} completed request {:?}", $host_shadow, ok));
        });
    }};
}
