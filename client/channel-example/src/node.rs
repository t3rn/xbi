use crate::manager::MessageManager;
use crate::Message;
use hex::ToHex;
use sp_core::crypto::Pair as PairExt;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::panic::catch_unwind;
use std::path::PathBuf;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{compose_extrinsic, Api, Metadata, PlainTipExtrinsicParams, XtStatus};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use xcm::latest::{
    AssetId, Instruction, Junction, Junctions, MultiAsset, MultiAssetFilter, MultiAssets,
    MultiLocation, OriginKind, WeightLimit, WildMultiAsset, Xcm,
};
use xcm::prelude::Fungible;
use xcm::{DoubleEncoded, VersionedMultiLocation, VersionedXcm};

#[derive(Debug, Clone)]
pub enum Command {
    Noop,
    Sudo(Vec<u8>),
    HrmpInitChannel,
    XbiSend(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub id: u64,
    pub host: String,
    pub sleep_time_secs: u64,
    pub key_pair: Option<PathBuf>,
}

impl NodeConfig {
    pub fn new(
        id: u64,
        host: String,
        sleep_time_secs: Option<u64>,
        key_pair: Option<PathBuf>,
    ) -> Self {
        NodeConfig {
            id,
            host,
            sleep_time_secs: sleep_time_secs.unwrap_or(5),
            key_pair,
        }
    }

    pub fn read_key_or_alice(&self) -> Pair {
        self.key_pair
            .clone()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|contents| Pair::from_string(&contents, None).ok())
            .unwrap_or_else(|| AccountKeyring::Alice.pair())
    }
}

impl MessageManager<Command> for NodeConfig {
    fn start(&self, mut rx: Receiver<Command>, _tx: Sender<Message>) {
        log::info!(
            "Starting node manager for id {} and host {}",
            self.id,
            self.host
        );

        let host_shadow = self.host.clone();
        let sleep_shadow = self.sleep_time_secs;
        let key_pair_shadow = self.read_key_or_alice();

        let _ = tokio::spawn(async move {
            let client = WsRpcClient::new(&host_shadow);
            let api = Api::<Pair, _, PlainTipExtrinsicParams>::new(client)
                .map(|api| api.set_signer(key_pair_shadow))
                .expect("Failed to initiate the rpc client");

            let meta: Option<Metadata> = api
                .get_metadata()
                .ok()
                .and_then(|meta| meta.try_into().ok());

            if let Some(meta) = meta {
                log::info!("{host_shadow} exposed {} pallets", meta.pallets.len());
                log::info!("{host_shadow} exposed {} events", meta.events.len());
                log::info!("{host_shadow} exposed {} errors", meta.errors.len());
                // TODO: assert capabilities on parachain, ensuring they have X installed
            }

            while let Some(msg) = rx.recv().await {
                use Command::*;
                log::debug!("Received request: {:?}", msg);
                // TODO: make requests to the node
                match msg {
                    Noop => {}
                    XbiSend(bytes) =>
                    // compose_extrinsic panics if the call is to something not in the metadata.
                    // Whilst ok for some, we don't want to have to restart the manager just because of some bad request
                    {
                        let extrinsic =
                            catch_unwind(|| compose_extrinsic!(api.clone(), "XBI", "send", bytes))
                                .ok();
                        if let Some(extrinsic) = extrinsic {
                            // FIXME: this blocks, use spawn_blocking

                            let _ = api
                                .send_extrinsic(extrinsic.hex_encode(), XtStatus::InBlock)
                                .map_err(|err| {
                                    log::error!("{host_shadow} failed to send request {:?}", err)
                                });
                        }
                    }
                    Sudo(bytes) => {
                        log::info!("sending sudo call {:?}", bytes);
                        let extrinsic =
                            catch_unwind(|| crate::extrinsic::sudo::wrap_sudo(api.clone(), bytes))
                                .ok();
                        if let Some(extrinsic) = extrinsic {
                            // FIXME: this blocks, use spawn_blocking

                            let _ = api
                                .send_extrinsic(extrinsic.hex_encode(), XtStatus::InBlock)
                                .map_err(|err| {
                                    log::error!("{host_shadow} failed to send request {:?}", err)
                                });
                        }
                    }
                    HrmpInitChannel => {
                        let dest = VersionedMultiLocation::V1(MultiLocation {
                            parents: 1,
                            interior: Junctions::Here,
                        });

                        let mut xcm: Xcm<Vec<u8>> = Xcm::new();
                        xcm.0
                            .push(Instruction::WithdrawAsset(MultiAssets::from(vec![
                                MultiAsset {
                                    id: AssetId::Concrete(MultiLocation {
                                        parents: 0,
                                        interior: Junctions::Here,
                                    }),
                                    fun: Fungible(1000000000000),
                                },
                            ])));
                        xcm.0.push(Instruction::BuyExecution {
                            fees: MultiAsset {
                                id: AssetId::Concrete(MultiLocation {
                                    parents: 0,
                                    interior: Junctions::Here,
                                }),
                                fun: Fungible(1000000000000),
                            },
                            weight_limit: WeightLimit::Unlimited,
                        });
                        // TODO: no
                        let call: DoubleEncoded<Vec<u8>> =
                            <DoubleEncoded<_> as From<Vec<u8>>>::from(
                                hex::decode("1700d10700000800000000a00f00").unwrap(),
                            );
                        xcm.0.push(Instruction::Transact {
                            origin_type: OriginKind::Native,
                            require_weight_at_most: 1000000000,
                            call,
                        });
                        xcm.0.push(Instruction::RefundSurplus);
                        xcm.0.push(Instruction::DepositAsset {
                            assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                            max_assets: 1,
                            beneficiary: MultiLocation {
                                parents: 0,
                                interior: Junctions::X1(Junction::Parachain(2001)),
                            },
                        });
                        let call = crate::extrinsic::xcm::xcm_send(
                            api.clone(),
                            dest,
                            VersionedXcm::V2(xcm),
                        );
                        let extrinsic =
                            catch_unwind(|| crate::extrinsic::sudo::wrap_sudo(api.clone(), call))
                                .ok();
                        if let Some(extrinsic) = extrinsic {
                            // FIXME: this blocks, use spawn_blocking

                            let _ = api
                                .send_extrinsic(extrinsic.hex_encode(), XtStatus::InBlock)
                                .map_err(|err| {
                                    log::error!("{host_shadow} failed to send request {:?}", err)
                                });
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(sleep_shadow)).await;
            }
        });
    }
}
