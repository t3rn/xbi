use crate::manager::MessageManager;
use crate::Message;
use hex::FromHex;
use serde::Deserialize;
use sp_core::sr25519;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{Api, EventsDecoder, Metadata, PlainTipExtrinsicParams, Raw, RawEvent};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

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

#[derive(Debug, Clone, Deserialize)]
pub struct ListenerEvent {
    pub module: String,
    pub variant: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriberNodeConfig {
    pub id: u64,
    pub host: String,
    pub sleep_time_secs: u64,
    pub listener_events: Vec<ListenerEvent>,
}

impl SubscriberNodeConfig {}

impl MessageManager<()> for SubscriberNodeConfig {
    fn start(&self, mut _rx: Receiver<()>, tx: Sender<Message>) {
        log::info!(
            "Starting subscriber manager for id {} at {} on events {:?}",
            self.id,
            self.host,
            self.listener_events
        );

        let id_shadow = self.id;
        let host_shadow = self.host.clone();
        let sleep_shadow = self.sleep_time_secs;
        let listener_events_shadow = self.listener_events.clone();

        // If we want to listen to events, we spawn a listener for each event
        if !listener_events_shadow.is_empty() {
            let _ = tokio::spawn(async move {
                let client = WsRpcClient::new(&host_shadow);
                let api = Api::<sr25519::Pair, _, PlainTipExtrinsicParams>::new(client).unwrap();
                let event_decoder = EventsDecoder::new(api.metadata.clone());

                let meta: Option<Metadata> = api
                    .get_metadata()
                    .ok()
                    .and_then(|meta| meta.try_into().ok());

                if let Some(meta) = meta {
                    log::info!("{host_shadow} exposed {} pallets", meta.pallets.len());
                    log::info!("{host_shadow} exposed {} events", meta.events.len());
                    log::info!("{host_shadow} exposed {} errors", meta.errors.len());
                }

                let (events_in, events_out) = std::sync::mpsc::channel();

                api.subscribe_events(events_in).unwrap();

                // TODO: remove unwraps
                // TODO: blocking_rt
                tokio::spawn(async move {
                    loop {
                        let events = events_out.recv().unwrap();
                        let events_str = events.strip_prefix("0x").unwrap_or(events.as_str());
                        log::debug!("{host_shadow} Found events of len {:?}", events_str.len());
                        let events = event_decoder
                            .decode_events(&mut Vec::from_hex(events_str).unwrap().as_slice());
                        match events {
                            Ok(raw_events) => {
                                for (phase, event) in raw_events.into_iter() {
                                    log::trace!(
                                        "{host_shadow} Decoded Event: {:?}, {:?}",
                                        phase,
                                        event
                                    );
                                    let event = match event {
                                        Raw::Event(raw)
                                            if listener_events_shadow.iter().any(|x| {
                                                x.module == raw.pallet && x.variant == raw.variant
                                            }) =>
                                        {
                                            log::info!(
                                                "{host_shadow} Found a tracked event {:?}",
                                                raw
                                            );
                                            Some(raw)
                                        }
                                        Raw::Error(runtime_error) => {
                                            log::debug!(
                                                "{host_shadow} Extrinsic Failed: {:?}",
                                                runtime_error
                                            );
                                            None
                                        }
                                        _ => {
                                            log::trace!(
                                                "{host_shadow} ignoring unsupported module event: {:?}",
                                                event
                                            );
                                            None
                                        }
                                    };
                                    if let Some(event) = event {
                                        tx.send(Message::SubscriberEvent(SubscriberEvent::new(
                                            id_shadow, event,
                                        )))
                                        .await
                                        .unwrap();
                                    }
                                }
                            }
                            Err(error) => {
                                log::error!("couldn't decode event record list: {:?}", error)
                            }
                        }

                        tokio::time::sleep(tokio::time::Duration::from_secs(sleep_shadow)).await;
                    }
                });
            });
        } else {
            log::info!("No listener events configured, stopping..")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
