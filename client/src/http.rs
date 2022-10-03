use crate::{Command, Message};
use actix_web::web::Data;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum HttpNodeCommand {
    Primary(Command),
    Secondary(Command),
}

impl HttpNodeCommand {
    pub fn get_command(&self) -> &Command {
        match self {
            HttpNodeCommand::Primary(cmd) => cmd,
            HttpNodeCommand::Secondary(cmd) => cmd,
        }
    }
}
/// A message which will inform the client what extrinsic to call on the node
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EncodedCodecMessage {
    pub kind: HttpNodeCommand,
}

pub fn setup_http_pipeline(global_sender: Arc<Sender<Message>>) {
    log::info!("Starting HTTP pipeline");
    tokio::spawn(
        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(global_sender.clone())) // add shared state
                .app_data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
                .wrap(middleware::Logger::default())
                .service(node_message)
        })
        .bind(("127.0.0.1", 8500))
        .expect("Failed to start HTTP server")
        .run(),
    );
}

#[actix_web::post("/")]
async fn node_message(global_sender: Data<Arc<Sender<Message>>>, body: web::Bytes) -> HttpResponse {
    fn insert_command(cmd: Command, msg_kind: HttpNodeCommand) -> Message {
        match msg_kind {
            HttpNodeCommand::Primary(_) => Message::PrimaryNode(cmd),
            HttpNodeCommand::Secondary(_) => Message::SecondaryNode(cmd),
        }
    }
    match serde_json::from_slice::<EncodedCodecMessage>(body.as_ref()) {
        Ok(msg) => match msg.kind.get_command() {
            Command::Sudo(bytes) => {
                log::debug!("Sending {:?} for dispatch", msg);
                global_sender
                    .send(insert_command(Command::Sudo(bytes.to_owned()), msg.kind))
                    .await
                    .map(|_| HttpResponse::Ok())
                    .unwrap_or_else(|_| HttpResponse::InternalServerError())
            }
            Command::HrmpInitChannel(parachain) => global_sender
                .send(insert_command(
                    Command::HrmpInitChannel(*parachain),
                    msg.kind,
                ))
                .await
                .map(|_| HttpResponse::Ok())
                .unwrap_or_else(|_| HttpResponse::InternalServerError()),
            Command::HrmpAcceptChannel(parachain) => global_sender
                .send(insert_command(
                    Command::HrmpAcceptChannel(*parachain),
                    msg.kind,
                ))
                .await
                .map(|_| HttpResponse::Ok())
                .unwrap_or_else(|_| HttpResponse::InternalServerError()),
            Command::UpdateRelayChain(new_host) => global_sender
                .send(insert_command(
                    Command::UpdateRelayChain(new_host.to_string()),
                    msg.kind,
                ))
                .await
                .map(|_| HttpResponse::Ok())
                .unwrap_or_else(|_| HttpResponse::InternalServerError()),
        },
        Err(_) => HttpResponse::BadRequest(),
    }
    .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_command() {
        let p = HttpNodeCommand::Primary(Command::HrmpAcceptChannel(300));
        let s = HttpNodeCommand::Secondary(Command::HrmpAcceptChannel(300));

        let p = serde_json::to_string(&EncodedCodecMessage { kind: p }).unwrap();
        let s = serde_json::to_string(&EncodedCodecMessage { kind: s }).unwrap();
        println!("{:?}", p);
        println!("{:?}", s);
        assert_ne!(p, s);
    }
}
