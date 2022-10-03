extern crate core;

use crate::config::Config;
use crate::manager::MessageManager;
use crate::node::{Command, NodeConfig};
use crate::subscriber::SubscriberNodeConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod config;
mod extrinsic;
#[cfg(feature = "webapi")]
mod http;
mod manager;
mod node;
mod subscriber;
#[derive(Debug, Clone)]
pub enum Message {
    PrimaryNode(Command),
    SecondaryNode(Command),
    SubscriberEvent(subscriber::SubscriberEvent),
}

// #[tokio::main(flavor = "multi_thread")]
#[tokio::main]
async fn main() {
    let config = Config::from_args();
    // TODO: overwrite initial config with args
    if config.debug {
        std::env::set_var(
            "RUST_LOG",
            // "substrate_api_client=none,xbi_client_channel_example=debug",
            "substrate_api_client=error,xbi_client_channel_example=info,xbi_client_channel_example::http=debug",
        );
    }
    env_logger::init();

    // Setup dispatch channel
    let (global_tx, mut global_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(256);

    // Setup primary node channel
    let (node_tx, node_rx) = mpsc::channel(256);
    NodeConfig::new(
        config.primary_parachain_id,
        config.primary_node_host,
        None,
        config.primary_node_seed,
    )
    .start(node_rx, global_tx.clone());

    // Setup secondary node channel
    let mut secondary_node_tx = None;
    if let Some(secondary_node_host) = config.secondary_node_host {
        let (node_tx, node_rx) = mpsc::channel(256);
        NodeConfig::new(
            config.secondary_parachain_id,
            secondary_node_host,
            None,
            config.secondary_node_seed,
        )
        .start(node_rx, global_tx.clone());
        secondary_node_tx = Some(node_tx);
    };

    // Setup each subscriber
    let subscribers: Vec<SubscriberNodeConfig> =
        serde_json::from_str(&config.subscribers).unwrap_or_default();
    for subscriber in subscribers {
        // Dummy channel since each subscriber doesnt handle messages, only send to the global dispatch.
        let (_dummy_tx, dummy_rx): (Sender<()>, Receiver<()>) = mpsc::channel(1);
        subscriber.start(dummy_rx, global_tx.clone())
    }

    #[cfg(feature = "webapi")]
    http::setup_http_pipeline(Arc::new(global_tx.clone()));

    let running = Arc::new(AtomicBool::new(true));
    setup_exit_handler(running.clone());

    // Init the dispatch loop, this blocks.
    let _main: Result<(), anyhow::Error> = tokio::spawn(async move {
        while let Some(msg) = global_rx.recv().await {
            if !running.load(Ordering::SeqCst) {
                log::info!("Exiting");
                continue;
            }
            match msg {
                Message::PrimaryNode(req) => {
                    let _ = node_tx
                        .send(req)
                        .await
                        .map_err(|e| log::error!("Failed to send command {}", e));
                }
                Message::SecondaryNode(req) => {
                    if let Some(node_tx) = &secondary_node_tx {
                        let _ = node_tx
                            .send(req)
                            .await
                            .map_err(|e| log::error!("Failed to send command {}", e));
                    }
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

fn setup_exit_handler(running: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        running.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
