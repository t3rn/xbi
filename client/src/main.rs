extern crate core;

use crate::config::Config;
use crate::manager::MessageManager;
use crate::node::{Command, NodeConfig};
use crate::subscriber::SubscriberConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    NNode(usize, Command),
    SubscriberEvent(subscriber::SubscriberEvent),
}

// #[tokio::main(flavor = "multi_thread")]
#[tokio::main]
async fn main() {
    let config = Config::new().apply_cli_args();
    env_logger::init();

    // Setup dispatch channel
    let (global_tx, mut global_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(256);

    let mut node_transmitters = vec![];
    for node in config.nodes.0 {
        let (node_tx, node_rx) = mpsc::channel(256);
        node.start(node_rx, global_tx.clone());
        node_transmitters.push(node_tx);
    }

    // Setup each subscriber
    for subscriber in config.subscribers.0 {
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
                    let _ = node_transmitters[0]
                        .send(req)
                        .await
                        .map_err(|e| log::error!("Failed to send command {}", e));
                }
                Message::SecondaryNode(req) => {
                    if let Some(node_tx) = node_transmitters.get(1) {
                        let _ = node_tx
                            .send(req)
                            .await
                            .map_err(|e| log::error!("Failed to send command {}", e));
                    }
                }
                Message::NNode(index, req) => {
                    if let Some(node_tx) = node_transmitters.get(index) {
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
