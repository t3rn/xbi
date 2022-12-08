use crate::manager::MessageManager;
use crate::Message;
use anyhow::Context;
use hex::FromHex;
use serde::{Deserialize, Serialize};
use sp_core::sr25519;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, EventsDecoder, Metadata, PlainTipExtrinsicParams, Raw, RawEvent};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

// Unused for now, useful when we start to actually action the events, not just log
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SubscriberEvent {
    id: u64,
    event: RawEvent,
}

impl SubscriberEvent {
    fn new(id: u64, event: RawEvent) -> SubscriberEvent {
        SubscriberEvent { id, event }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Subscription {
    pub module: String,
    pub variant: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriberConfig {
    pub parachain_id: u64,
    pub host: String,
    pub sleep_time_secs: u64,
    pub listener_events: Vec<Subscription>,
}

impl MessageManager<()> for SubscriberConfig {
    // Since subscriber is pull, it doesnt handle requests
    fn start(&self, mut _rx: Receiver<()>, tx: Sender<Message>) -> anyhow::Result<()> {
        log::info!(
            "Starting subscriber manager for id {} at {} on events {:?}",
            self.parachain_id,
            self.host,
            self.listener_events
        );

        let id_shadow = self.parachain_id;
        let host_shadow = self.host.clone();
        let sleep_shadow = self.sleep_time_secs;
        let listener_events_shadow = self.listener_events.clone();

        if !listener_events_shadow.is_empty() {
            tokio::spawn(async move {
                let client = WsRpcClient::new(&host_shadow);
                let api = Api::<sr25519::Pair, _, PlainTipExtrinsicParams>::new(client)
                    .context("{host_shadow} Failed to initialize the api")?;
                let event_decoder = EventsDecoder::new(api.metadata.clone());

                let meta: Metadata = api
                    .get_metadata()
                    .ok()
                    .and_then(|meta| meta.try_into().ok())
                    .context("Failed to read context")?;

                log::info!("{host_shadow} exposed {} pallets", meta.pallets.len());
                log::info!("{host_shadow} exposed {} events", meta.events.len());
                log::info!("{host_shadow} exposed {} errors", meta.errors.len());

                let (events_in, events_out) = std::sync::mpsc::channel();
                api.subscribe_events(events_in)?;

                let tx_shadow = tx.clone();
                tokio::spawn(async move {
                    loop {
                        let events_out_shadow = events_out.recv();
                        if let Ok(events) = events_out_shadow {
                            let events = events.trim_start_matches("0x").to_string();

                            for (phase, event) in event_decoder
                                .decode_events(&mut Vec::from_hex(events).unwrap().as_slice())
                                .ok()
                                .context("{host_shadow} Failed to decode events")?
                                .into_iter()
                            {
                                log::trace!(
                                    "{host_shadow} Decoded Event: {:?}, {:?}",
                                    phase,
                                    event
                                );
                                match event {
                                    Raw::Event(raw)
                                        if listener_events_shadow.iter().any(|x| {
                                            x.module == raw.pallet && x.variant == raw.variant
                                        }) =>
                                    {
                                        log::debug!(
                                            "{host_shadow} Found a tracked event {:?}",
                                            raw
                                        );
                                        let _ = tx_shadow
                                            .send(Message::SubscriberEvent(SubscriberEvent::new(
                                                id_shadow, raw,
                                            )))
                                            .await;
                                    }
                                    Raw::Error(runtime_error) => {
                                        log::debug!(
                                            "{host_shadow} Extrinsic Failed: {:?}",
                                            runtime_error
                                        );
                                    }
                                    _ => {
                                        log::trace!(
                                            "{host_shadow} ignoring unsupported module event: {:?}",
                                            event
                                        );
                                    }
                                }
                            }

                            tokio::time::sleep(tokio::time::Duration::from_secs(sleep_shadow))
                                .await;
                        } else {
                            break;
                        }
                    }

                    Ok::<(), anyhow::Error>(())
                });

                Ok::<(), anyhow::Error>(())
            });
        } else {
            log::info!("{host_shadow} No listener events configured, stopping..")
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
