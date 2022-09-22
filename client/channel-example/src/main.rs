use crate::config::Config;
use codec::Decode;
use sp_core::sr25519;
use sp_runtime::AccountId32 as AccountId;
///! Very simple example that shows how to subscribe to events generically
/// implying no runtime needs to be imported
use std::sync::mpsc::channel;
use structopt::StructOpt;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, PlainTipExtrinsicParams};

mod config;

// Look at the how the transfer event looks like in in the metadata
#[derive(Decode)]
struct TransferEventArgs {
    from: AccountId,
    to: AccountId,
    value: u128,
}

fn main() {
    env_logger::init();
    let config = Config::from_args();

    if config.debug {
        std::env::set_var("RUST_LOG", "debug");
    }

    // extract initial config file
    // overwrite initial config with args
    // let url = get_node_url_from_cli();

    // setup manager for parachain A
    // register the handler
    let client = WsRpcClient::new(&build_url(&config));
    let api = Api::<sr25519::Pair, _, PlainTipExtrinsicParams>::new(client).unwrap();
    println!("Subscribe to events");
    let (events_in, events_out) = channel();

    api.subscribe_events(events_in).unwrap();
    let args: TransferEventArgs = api
        .wait_for_event("Balances", "Transfer", None, &events_out)
        .unwrap();

    // if parachain b
    // setup manager for parachain b

    // dispatch loop

    println!("Transactor: {:?}", args.from);
    println!("Destination: {:?}", args.to);
    println!("Value: {:?}", args.value);
}

fn build_url(config: &Config) -> String {
    let host = format!("{}://{}", config.node_protocol, config.node_host);
    log::debug!("Built URL: {}", host);
    host
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
