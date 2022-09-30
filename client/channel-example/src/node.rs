use crate::manager::MessageManager;
use crate::Message;
use sp_core::crypto::Pair as PairExt;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use std::panic::catch_unwind;
use std::path::PathBuf;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{compose_extrinsic, Api, Metadata, PlainTipExtrinsicParams, XtStatus};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum Command {
    Noop,
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
                let extrinsic = match msg {
                    Noop => None,
                    XbiSend(bytes) =>
                    // compose_extrinsic panics if the call is to something not in the metadata.
                    // Whilst ok for some, we don't want to have to restart the manager just because of some bad request
                    {
                        catch_unwind(|| compose_extrinsic!(api.clone(), "XBI", "send", bytes)).ok()
                    }
                };
                if let Some(extrinsic) = extrinsic {
                    // FIXME: this blocks, use spawn_blocking
                    let _ = api
                        .send_extrinsic(extrinsic.hex_encode(), XtStatus::InBlock)
                        .map_err(|err| {
                            log::error!("{host_shadow} failed to send request {:?}", err)
                        });
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(sleep_shadow)).await;
            }
        });
    }
}
