use crate::config::Config;
use crate::manager::MessageManager;
use crate::node::NodeConfig;
use crate::subscriber::SubscriberNodeConfig;
use codec::Decode;
use sp_runtime::AccountId32 as AccountId;
///! Very simple example that shows how to subscribe to events generically
/// implying no runtime needs to be imported
use structopt::StructOpt;
use substrate_api_client::PlainTipExtrinsicParams;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod config;
mod manager;
mod node;
mod subscriber;

#[derive(Debug, Clone)]
pub enum Message {
    NodeRequest(node::NodeMessage),
    SubscriberEvent(subscriber::SubscriberEvent),
}

#[tokio::main]
async fn main() {
    let config = Config::from_args();
    if config.debug {
        std::env::set_var(
            "RUST_LOG",
            // "substrate_api_client=debug,xbi_client_channel_example=debug",
            "substrate_api_client=none,xbi_client_channel_example=debug",
        );
    }
    env_logger::init();
    // TODO: overwrite initial config with args

    let (global_tx, mut global_rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(256);

    let (node_tx, node_rx) = mpsc::channel(256);
    NodeConfig::new(
        config.primary_node_id,
        build_url(&config.primary_node_protocol, &config.primary_node_host),
        None,
    )
    .start(node_rx, global_tx.clone());

    let subscribers: Vec<SubscriberNodeConfig> =
        serde_json::from_str(&config.subscribers).unwrap_or_default();
    for subscriber in subscribers {
        let (_dummy_tx, dummy_rx): (Sender<()>, Receiver<()>) = mpsc::channel(1);
        subscriber.start(dummy_rx, global_tx.clone())
    }

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
                    log::info!("Watched event from subscriber: {:?}", event);
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    })
    .await
    .expect("Dispatch loop failed");
}

fn build_url(protocol: &String, host: &String) -> String {
    let host = format!("{}://{}", protocol, host);
    log::debug!("Built URL: {}", host);
    host
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
