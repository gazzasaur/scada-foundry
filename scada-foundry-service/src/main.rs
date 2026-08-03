pub mod api;
pub mod config;
mod context;
pub mod core;
pub mod error;
pub mod iccp;

use std::net::SocketAddr;

use anyhow::Error;
use axum::{
    Json, Router,
    extract::{
        ConnectInfo,
        ws::{WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, post},
};
use axum_extra::{TypedHeader, headers};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::{join, sync::mpsc::unbounded_channel};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};

use crate::{
    config::ApplicationConfiguration,
    iccp::{
        IccpSubsystem,
        api::IccpAssociation ,
    },
};

/// SCADA Foundry Server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file containing the server configuration
    #[arg(short, long, default_value = "config.json")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let app_config = ApplicationConfiguration::load(args.config_file.as_str()).await?;

    let (global_listener_sender, _global_listener_receiver) = unbounded_channel();

    let mut iccp_manager = IccpSubsystem::new(global_listener_sender).await;

    for iccp_association in app_config.iccp.associations {
        iccp_manager.create_association(iccp_association).await?;
    }

    let cors = CorsLayer::permissive();
    let app = Router::new()
        .route("/", get(root))
        .route("/app/api/createiccpassociation", post(create_user))
        .route("/app/ws", any(ws_handler))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let web_server_task = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>());
    let _ = join!(web_server_task);

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(Json(payload): Json<IccpAssociation>) -> (StatusCode, Json<IccpAssociation>) {
    (StatusCode::CREATED, Json(payload))
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

async fn ws_handler(ws: WebSocketUpgrade, user_agent: Option<TypedHeader<headers::UserAgent>>, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    // Finalize the upgrade handshake
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // Process messages here
    while let Some(Ok(msg)) = socket.recv().await {
        if socket.send(msg).await.is_err() {
            break; // Connection dropped
        }
    }
}
