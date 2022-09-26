use crate::{Message, NodeMessage};
use actix_web::web::Data;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

/// A message which will inform the client what extrinsic to call on the node
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EncodedCodecMessage {
    /// Provide to the deserializer what type the message is
    pub kind: String,
    // TODO: can be expanded to something less opaque
    #[serde(with = "hex::serde")]
    pub bytes: Vec<u8>,
}

pub async fn setup_http_pipeline(global_sender: Arc<Sender<Message>>) {
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
    match serde_json::from_slice::<EncodedCodecMessage>(body.as_ref()) {
        // TODO: move to resolver somewhere
        Ok(msg) if &msg.kind == "XbiFormat" => {
            log::debug!("Sending {:?} for dispatch", msg);
            global_sender
                .send(Message::NodeRequest(NodeMessage::XbiSend(msg.bytes)))
                .await
                .map(|_| HttpResponse::Ok().finish())
                .unwrap_or(HttpResponse::InternalServerError().finish())
        }
        Ok(msg) => {
            log::debug!("Unsupported message type {:?}", msg);
            HttpResponse::ExpectationFailed().finish()
        }
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}
