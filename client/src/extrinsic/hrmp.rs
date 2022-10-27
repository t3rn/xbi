use crate::extrinsic::index_from_metadata;
use codec::Encode;
use sp_core::sr25519::Pair;
use sp_keyring::AccountKeyring;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, Metadata, PlainTipExtrinsicParams};

const HRMP_PALLET: &str = "Hrmp";
const HRMP_INIT_CHANNEL: &str = "hrmp_init_open_channel";
const HRMP_ACCEPT_CHANNEL: &str = "hrmp_accept_open_channel";

const ROCOCO_RELAYCHAIN_OFFICIAL_HOST: &str = "wss://rococo-rpc.polkadot.io";

#[memoize::memoize]
pub fn get_relaychain_metadata(host: Option<String>) -> Metadata {
    let pair = AccountKeyring::Alice.pair();

    let host = host.unwrap_or_else(|| ROCOCO_RELAYCHAIN_OFFICIAL_HOST.to_string());
    let client = WsRpcClient::new(&host);
    let api = Api::<Pair, _, PlainTipExtrinsicParams>::new(client)
        .map(|api| api.set_signer(pair))
        .expect("Failed to initiate the rpc client");

    api.get_metadata()
        .ok()
        .and_then(|meta| meta.try_into().ok())
        .expect("Failed to read metadata from relaychain")
}

pub fn init_open_channel_req(
    parachain_id: u32,
    proposed_max_capacity: Option<u32>,
    proposed_max_message_size: Option<u32>,
    relaychain_meta: Option<Metadata>,
) -> Vec<u8> {
    let pallet_and_call = if let Some(relaychain_meta) = relaychain_meta {
        let (pallet_idx, call_idx) = index_from_metadata(
            relaychain_meta,
            HRMP_PALLET.to_string(),
            HRMP_INIT_CHANNEL.to_string(),
        )
        .unwrap();
        [pallet_idx, call_idx]
    } else {
        [23, 0] // roco
    }
    .to_vec();
    [
        pallet_and_call,
        parachain_id.encode(),
        proposed_max_capacity.unwrap_or(8).encode(),
        proposed_max_message_size.unwrap_or(1024).encode(),
    ]
    .concat()
}

pub fn accept_channel_req(parachain_id: u32, relaychain_meta: Option<Metadata>) -> Vec<u8> {
    let pallet_and_call = if let Some(relaychain_meta) = relaychain_meta {
        let (pallet_idx, call_idx) = index_from_metadata(
            relaychain_meta,
            HRMP_PALLET.to_string(),
            HRMP_ACCEPT_CHANNEL.to_string(),
        )
        .unwrap();
        [pallet_idx, call_idx]
    } else {
        [23, 1] // roco
    }
    .to_vec();
    [
        pallet_and_call, // call_index
        parachain_id.encode(),
    ]
    .concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let expected = hex::decode("1700d10700000800000000a00f00").unwrap();
        assert_eq!(
            init_open_channel_req(2001, None, Some(1024000), None),
            expected
        )
    }
    #[test]
    fn test_accept() {
        let expected = hex::decode("1701b80b0000").unwrap();
        assert_eq!(accept_channel_req(3000, None), expected)
    }
}
