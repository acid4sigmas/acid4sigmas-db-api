use crate::db::db_handler::DatabaseHandler;
use acid4sigmas_models::error_response;
use acid4sigmas_models::models::auth::AuthTokens;
use acid4sigmas_models::models::db::DatabaseRequest;
use acid4sigmas_models::models::db::DatabaseResponse;
use acid4sigmas_models::secrets::init_secrets;
use acid4sigmas_models::secrets::SECRET_KEY;
use acid4sigmas_models::utils::jwt::BackendClaims;
use actix_web::{get, rt, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::AggregatedMessage;
use db::db_handler::DbHandler;
use futures_util::StreamExt as _;

use std::path::PathBuf;
use tokio::time::sleep;
use tokio::time::Duration;

use acid4sigmas_models::utils::jwt::JwtToken;

mod cache;
mod db;

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
        let jwt_token = JwtToken::new(SECRET_KEY.get().unwrap());

        match jwt_token.decode_jwt::<BackendClaims>(&token) {
            Ok(_) => (),
            Err(e) => {
                println!("{:?}", e);
                return Ok(error_response!(403, e.to_string()));
            }
        }
    } else {
        return Ok(error_response!(403, "no token found."));
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
                    match serde_json::from_str::<DatabaseRequest>(&text) {
                        Ok(mut request) => {
                            if let Err(e) = request.validate() {
                                let error_message: DatabaseResponse<serde_json::Value> =
                                    DatabaseResponse::Error {
                                        error: e.to_string(),
                                    };
                                let error_text = serde_json::to_string(&error_message).unwrap();
                                session.text(error_text).await.unwrap();
                                continue;
                            }

                            let db_handler_result = DatabaseHandler::new(request).await;

                            if let Err(e) = db_handler_result {
                                let error_message: DatabaseResponse<serde_json::Value> =
                                    DatabaseResponse::Error {
                                        error: e.to_string(),
                                    };
                                let error_text = serde_json::to_string(&error_message).unwrap();
                                session.text(error_text).await.unwrap();
                                continue;
                            }

                            let db_handler = db_handler_result.unwrap();

                            let db_handler_request_result = db_handler.handle_request().await;

                            if let Err(e) = db_handler_request_result {
                                let error_message: DatabaseResponse<serde_json::Value> =
                                    DatabaseResponse::Error {
                                        error: e.to_string(),
                                    };
                                let error_text = serde_json::to_string(&error_message).unwrap();
                                session.text(error_text).await.unwrap();
                                continue;
                            }

                            let db_handler_response = db_handler_request_result.unwrap();

                            // Serialize DatabaseResponse and send it
                            let response_text =
                                serde_json::to_string(&db_handler_response).unwrap();
                            session.text(response_text).await.unwrap();
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
                    println!("heartbeat received");
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

const INDEX_BODY: &str = include_str!("index.html");

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(INDEX_BODY)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = init_secrets("Secrets.toml"); // init all secrets
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

    HttpServer::new(|| App::new().route("/db", web::get().to(db_ws)).service(index))
        .bind(("127.0.0.1", 3453))?
        .run()
        .await
}

use acid4sigmas_models::db::ModelRegistry;
use acid4sigmas_models::models::{api::users::User, auth::AuthUser};
use std::sync::OnceLock;

pub static MODEL_REGISTRY: OnceLock<ModelRegistry> = OnceLock::new();

// dont forget to register your structs!
fn initialize_models() {
    let mut registry = ModelRegistry::new();

    registry.register::<User>();
    registry.register::<AuthUser>();
    registry.register::<AuthTokens>();

    MODEL_REGISTRY
        .set(registry)
        .expect("failed to set registry");
}
