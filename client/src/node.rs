use crate::extrinsic::hrmp::{accept_channel_req, get_relaychain_metadata, init_open_channel_req};
use crate::extrinsic::xcm::XcmBuilder;
use crate::manager::MessageManager;
use crate::{catch_panicable, Message};
use sp_core::crypto::Pair as PairExt;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::path::PathBuf;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata, PlainTipExtrinsicParams, XtStatus};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use xcm::VersionedXcm;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Command {
    #[serde(with = "hex::serde")]
    Sudo(Vec<u8>),
    HrmpInitChannel(u32),
    HrmpAcceptChannel(u32),
    UpdateRelayChain(String),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct NodeConfig {
    pub parachain_id: u32,
    pub host: String,
    pub sleep_time_secs: u64,
    pub key_seed: Option<PathBuf>,
}

impl NodeConfig {
    pub fn new(
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
            .and_then(|contents| {
                Pair::from_string(&contents.replace(' ', "/").trim().to_string(), None).ok()
            })
            .unwrap_or_else(|| AccountKeyring::Alice.pair())
    }
}

impl MessageManager<Command> for NodeConfig {
    fn start(&self, mut rx: Receiver<Command>, _tx: Sender<Message>) {
        log::info!("Starting node manager for host {}", self.host);

        let host_shadow = self.host.clone();
        let sleep_shadow = self.sleep_time_secs;
        let key_pair_shadow = self.read_key_or_alice();
        let id_shadow = self.parachain_id;
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

                        if let Some(extrinsic) =
                            catch_panicable!(crate::extrinsic::sudo::wrap_sudo(api.clone(), bytes))
                        {
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
                            .with_withdraw_asset(Some(0), 1000000000000)
                            .with_buy_execution(Some(0), 1000000000000)
                            .with_transact(
                                Some(1000000000),
                                init_open_channel_req(parachain, None, None, Some(relaychain_meta)),
                            )
                            .with_refund_surplus()
                            .with_deposit_asset(Some(0), 1, id_shadow)
                            .build();

                        let call = crate::extrinsic::xcm::xcm_send(
                            api.clone(),
                            XcmBuilder::get_relaychain_dest(),
                            VersionedXcm::V2(call),
                        );
                        if let Some(extrinsic) =
                            catch_panicable!(crate::extrinsic::sudo::wrap_sudo(api.clone(), call))
                        {
                            tokio::task::spawn_blocking(move || {
                                let _ = api_shadow
                                    .send_extrinsic(extrinsic.hex_encode(), XtStatus::InBlock)
                                    .map_err(|err| {
                                        log::error!(
                                        "{host_shadow} failed to send init channel request {:?}",
                                        err
                                    )
                                    })
                                    .map(|ok| {
                                        log::info!("{host_shadow} completed request {:?}", ok)
                                    });
                            });
                        }
                    }
                    HrmpAcceptChannel(parachain) => {
                        let relaychain_meta = get_relaychain_metadata(relaychain_host.clone());

                        let call = XcmBuilder::default()
                            .with_withdraw_asset(Some(0), 1000000000000)
                            .with_buy_execution(Some(0), 1000000000000)
                            .with_transact(
                                Some(1000000000),
                                accept_channel_req(parachain, Some(relaychain_meta)),
                            )
                            .with_refund_surplus()
                            .with_deposit_asset(Some(0), 1, id_shadow)
                            .build();

                        let call = crate::extrinsic::xcm::xcm_send(
                            api.clone(),
                            XcmBuilder::get_relaychain_dest(),
                            VersionedXcm::V2(call),
                        );
                        if let Some(extrinsic) =
                            catch_panicable!(crate::extrinsic::sudo::wrap_sudo(api.clone(), call))
                        {
                            tokio::task::spawn_blocking(move || {
                                let _ = api_shadow
                                    .send_extrinsic(extrinsic.hex_encode(), XtStatus::InBlock)
                                    .map_err(|err| {
                                        log::error!(
                                            "{host_shadow} failed to send request {:?}",
                                            err
                                        )
                                    })
                                    .map(|ok| {
                                        log::info!("{host_shadow} completed request {:?}", ok)
                                    });
                            });
                        }
                    }
                    UpdateRelayChain(new_host) => {
                        log::info!("{host_shadow} updating relay chain to {new_host}");
                        relaychain_host = Some(new_host);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(sleep_shadow)).await;
            }
        });
    }
}
