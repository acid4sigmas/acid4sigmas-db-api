use crate::db::db_handler::DatabaseHandler;
use acid4sigmas_models::error_response;
use acid4sigmas_models::models::db::DatabaseRequest;
use acid4sigmas_models::utils::jwt::BackendClaims;
use actix_web::{rt, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::AggregatedMessage;
use db::db_handler::DbHandler;
use futures_util::StreamExt as _;
use secrets::SECRETS;
use serde_json::json;
use std::path::PathBuf;
use tokio::time::sleep;
use tokio::time::Duration;

use acid4sigmas_models::utils::jwt::JwtToken;

mod cache;
mod db;
mod secrets;
mod timer;
mod tokio_spawner;

async fn db_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let query = req.query_string();

    let token: Option<String> =
        web::Query::<std::collections::HashMap<String, String>>::from_query(query)
            .unwrap()
            .get("token")
            .map(|t| t.to_string());

    if let Some(token) = token {
        let jwt_token = JwtToken::new(SECRETS.get("SECRET_KEY").unwrap());

        match jwt_token.decode_jwt::<BackendClaims>(&token) {
            Ok(_) => (),
            Err(e) => return Ok(error_response!(403, e.to_string())),
        }
    } else {
        println!("no token was provided");
        return Ok(HttpResponse::Unauthorized().body("no token found"));
    }

    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    if let Some(peer_addr) = req.peer_addr() {
        println!("New WebSocket connection established from: {}", peer_addr);
    } else {
        println!("New WebSocket connection established");
    }

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    println!("request: {:?}", text);
                    match serde_json::from_str::<DatabaseRequest>(&text) {
                        Ok(mut request) => {
                            if let Err(e) = request.validate() {
                                let error_message = json!({
                                    "error": e.to_string()
                                });
                                session.text(error_message.to_string()).await.unwrap();
                                continue;
                            }

                            let db_handler_result = DatabaseHandler::new(request).await;

                            if let Err(e) = db_handler_result {
                                let error_message = json!({
                                    "error": e.to_string()
                                });
                                session.text(error_message.to_string()).await.unwrap();
                                continue;
                            }

                            let db_handler = db_handler_result.unwrap();

                            let db_handler_request_result = db_handler.handle_request().await;

                            if let Err(e) = db_handler_request_result {
                                let error_message = json!({
                                    "error": e.to_string()
                                });
                                session.text(error_message.to_string()).await.unwrap();
                                continue;
                            }

                            let db_handler_request = db_handler_request_result.unwrap();

                            if let Some(value) = db_handler_request {
                                session
                                    .text(serde_json::to_string(&value).unwrap())
                                    .await
                                    .unwrap();
                            } else {
                                let success_message = json!({"status": "success"});
                                session.text(success_message.to_string()).await.unwrap();
                            }
                        }
                        Err(e) => {
                            let error_message = format!("Failed to parse request: {}", e);
                            session.text(error_message).await.unwrap();
                        }
                    }
                }

                Ok(AggregatedMessage::Binary(bin)) => {
                    // echo binary message
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    // respond to PING frame with PONG frame
                    session.pong(&msg).await.unwrap();
                }

                Ok(AggregatedMessage::Pong(msg)) => {
                    println!("Received Pong Message: {:?}", msg);
                }

                Ok(AggregatedMessage::Close(reason)) => {
                    println!("Received Close Message with reason: {:?}", reason);
                    break; // Exit the loop on close
                }

                Err(e) => {
                    println!("Error while processing message: {:?}", e);
                    break;
                }
            }
        }
    });

    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    initialize_models();

    tokio_spawner::TokioSpawner::spawn(async move {
        loop {
            match db::Database::init(PathBuf::from("schema.sql")).await {
                Ok(()) => (),
                Err(e) => eprintln!("error: {}", e),
            };

            sleep(Duration::from_secs(15)).await
        }
    });

    HttpServer::new(|| App::new().route("/db", web::get().to(db_ws)))
        .bind(("127.0.0.1", 3453))?
        .run()
        .await
}

use acid4sigmas_models::db::ModelRegistry;
use acid4sigmas_models::models::api::users::User;
use std::sync::OnceLock;

pub static MODEL_REGISTRY: OnceLock<ModelRegistry> = OnceLock::new();

fn initialize_models() {
    let mut registry = ModelRegistry::new();

    registry.register::<User>();

    MODEL_REGISTRY
        .set(registry)
        .expect("failed to set registry");
}
