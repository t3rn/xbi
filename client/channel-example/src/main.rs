extern crate core;

use crate::config::Config;
use crate::manager::MessageManager;
use crate::node::{NodeConfig, NodeMessage};
use crate::subscriber::SubscriberNodeConfig;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod config;
#[cfg(feature = "webapi")]
mod http;
mod manager;
mod node;
mod subscriber;

#[derive(Debug, Clone)]
pub enum Message {
    NodeRequest(NodeMessage),
    SubscriberEvent(subscriber::SubscriberEvent),
}

#[tokio::main]
async fn main() {
    let config = Config::from_args();
    if config.debug {
        std::env::set_var(
            "RUST_LOG",
            // "substrate_api_client=none,xbi_client_channel_example=debug",
            "substrate_api_client=error,xbi_client_channel_example=info,xbi_client_channel_example::http=debug",
        );
    }
    env_logger::init();
    // TODO: overwrite initial config with args

    // Setup dispatch channel
    let (global_tx, mut global_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(256);

    // Setup primary node channel
    let (node_tx, node_rx) = mpsc::channel(256);
    NodeConfig::new(config.primary_node_id, config.primary_node_host, None, None)
        .start(node_rx, global_tx.clone());

    // Setup each subscriber
    let subscribers: Vec<SubscriberNodeConfig> =
        serde_json::from_str(&config.subscribers).unwrap_or_default();
    for subscriber in subscribers {
        // Dummy channel since each subscriber doesnt handle messages, only send to the global dispatch.
        let (_dummy_tx, dummy_rx): (Sender<()>, Receiver<()>) = mpsc::channel(1);
        subscriber.start(dummy_rx, global_tx.clone())
    }

    #[cfg(feature = "webapi")]
    http::setup_http_pipeline(Arc::new(global_tx.clone())).await;

    // Init the dispatch loop, this blocks.
    let _main: Result<(), anyhow::Error> = tokio::spawn(async move {
        while let Some(msg) = global_rx.recv().await {
            match msg {
                Message::NodeRequest(req) => {
                    let _ = node_tx
                        .send(req)
                        .await
                        .map_err(|e| log::error!("Failed to send command {}", e));
                }
                Message::SubscriberEvent(event) => {
                    log::trace!("Watched event from subscriber: {:?}", event);
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    })
    .await
    .expect("Dispatch loop failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
