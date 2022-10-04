extern crate core;

use crate::config::Config;
use crate::manager::MessageManager;
use crate::node::{Command, NodeConfig};
use crate::subscriber::SubscriberConfig;
use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook::iterator::Handle;
use signal_hook_tokio::Signals;
use std::process::exit;
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
    Kill,
    PrimaryNode(Command),
    SecondaryNode(Command),
    NNode(usize, Command),
    SubscriberEvent(subscriber::SubscriberEvent),
}

// #[tokio::main(flavor = "multi_thread")]
#[tokio::main]
async fn main() {
    let config = Config::new().apply_cli_args();
    pretty_env_logger::init();
    log::debug!("Deserialized config: {:?}", config);

    // Setup dispatch channel
    let (global_tx, mut global_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(256);

    let mut node_transmitters = vec![];
    for node in config.nodes {
        let (node_tx, node_rx) = mpsc::channel(256);
        node.start(node_rx, global_tx.clone());
        node_transmitters.push(node_tx);
    }

    // Setup each subscriber
    for subscriber in config.subscribers {
        // Dummy channel since each subscriber doesnt handle messages, only send to the global dispatch.
        let (_dummy_tx, dummy_rx): (Sender<()>, Receiver<()>) = mpsc::channel(1);
        subscriber.start(dummy_rx, global_tx.clone())
    }

    let global_tx = Arc::new(global_tx.clone());
    #[cfg(feature = "webapi")]
    http::setup_http_pipeline(global_tx.clone());

    setup_exit_handler(global_tx.clone());

    // Init the dispatch loop, this blocks.
    let _main: Result<(), anyhow::Error> = tokio::spawn(async move {
        while let Some(msg) = global_rx.recv().await {
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
                Message::Kill => {
                    log::info!("Terminating");
                    exit(0);
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    })
    .await
    .expect("Dispatch loop failed");
}

async fn handle_signals(global_tx: Arc<Sender<Message>>, mut signals: Signals, handle: Handle) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                // Reload configuration
                // Reopen the log file
            }
            SIGTERM | SIGINT | SIGQUIT => {
                log::debug!("Received exit signal: {:?}, sending kill message", signal);
                let tx_shadow = global_tx.clone();
                if let Err(err) = tx_shadow.send(Message::Kill).await {
                    log::warn!("Failed to send kill message {:?}", err);
                }
                // Terminate the signal stream.
                handle.close();
            }
            _ => {}
        }
    }
}

fn setup_exit_handler(global_tx: Arc<Sender<Message>>) {
    let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT]).unwrap();
    let handle = signals.handle();

    tokio::spawn(handle_signals(global_tx, signals, handle));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
