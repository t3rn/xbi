use crate::manager::MessageManager;
use crate::Message;
use sp_core::crypto::Pair as PairExt;
use sp_core::{sr25519, sr25519::Pair};
use sp_keyring::AccountKeyring;
use std::path::PathBuf;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, PlainTipExtrinsicParams};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum NodeMessage {
    Noop,
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
            .unwrap_or(AccountKeyring::Alice.pair())
    }
}

impl MessageManager<NodeMessage> for NodeConfig {
    fn start(&self, mut rx: Receiver<NodeMessage>, tx: Sender<Message>) {
        log::debug!(
            "Starting node manager for id {} and host {}",
            self.id,
            self.host
        );

        let host_shadow = self.host.clone();
        let sleep_shadow = self.sleep_time_secs.clone();
        let key_pair_shadow = self.read_key_or_alice();

        let _ = tokio::spawn(async move {
            let client = WsRpcClient::new(&host_shadow);
            let _api = Api::<Pair, _, PlainTipExtrinsicParams>::new(client)
                .map(|api| api.set_signer(key_pair_shadow))
                .unwrap();

            while let Some(msg) = rx.recv().await {
                use NodeMessage::*;
                match msg {
                    Noop => {}
                }
                // TODO: make requests to the node
                log::debug!("Received request: {:?}", msg);
                tokio::time::sleep(tokio::time::Duration::from_secs(sleep_shadow)).await;
            }
        });
    }
}
