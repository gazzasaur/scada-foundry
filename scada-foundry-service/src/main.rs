pub mod api;
pub mod config;
mod context;
pub mod core;
pub mod error;
pub mod iccp;
pub mod webservice;

use std::{collections::VecDeque, net::SocketAddr, sync::Arc};

use anyhow::Error;
use axum::{
    Json, Router,
    extract::{
        ConnectInfo, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, post},
};
use axum_extra::{TypedHeader, headers};
use clap::Parser;
use futures::stream::FuturesUnordered;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use tokio::{
    join, select,
    sync::{
        Mutex, RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    },
};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{error, warn};
use uuid::{Uuid, uuid};

use crate::{
    api::ScadaFoundryEvent::{self, IccpAssociationUpdate, IccpDataPointUpdate},
    config::ApplicationConfiguration,
    iccp::{
        IccpSubsystem,
        api::{IccpAssociation, IccpAssociationState},
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

#[derive(Clone)]
struct WebAppContext {
    pub config: Arc<RwLock<ApplicationConfiguration>>,
    pub iccp_subsystem: Arc<RwLock<IccpSubsystem>>,
    pub stream_queue_sender: UnboundedSender<UnboundedSender<ScadaFoundryEvent>>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let app_config = Arc::new(RwLock::new(ApplicationConfiguration::load(args.config_file.as_str()).await?));

    let (stream_queue_sender, mut stream_queue_receiver) = unbounded_channel();
    let (global_listener_sender, mut global_listener_receiver) = unbounded_channel();

    let iccp_manager = Arc::new(RwLock::new(IccpSubsystem::new(global_listener_sender).await));
    let web_app_context = WebAppContext { iccp_subsystem: iccp_manager.clone(), config: app_config.clone(), stream_queue_sender };

    // TODO This lazily removes queues when events occur.
    // Can also call 'closed' on all the queues to remove them immediately after they are closed.
    tokio::task::spawn(async move {
        let mut queues = VecDeque::new();
        loop {
            tokio::select! {
                x = global_listener_receiver.recv() => match x {
                    Some(x) => {
                        queues = queues.drain(..).into_iter().filter_map(|q: UnboundedSender<ScadaFoundryEvent>| {
                            match q.send(x.clone()) {
                                Ok(()) => Some(q),
                                Err(_) => None,
                            }
                        }).collect();
                    },
                    None => (),
                },
                x = stream_queue_receiver.recv() => match x {
                    Some(x) => queues.push_back(x),
                    None => return,
                }
            }
        }
    });

    for iccp_association in app_config.write().await.iccp.associations.iter().cloned() {
        iccp_manager.write().await.create_association(iccp_association).await;
    }

    let cors = CorsLayer::permissive();
    let app = Router::new()
        .route("/", get(root))
        .route("/app/api/fetchiccpassociations", get(fetch_iccp_associations))
        .route("/app/api/createiccpassociation", post(create_iccp_association))
        .route("/app/ws", any(ws_handler))
        .with_state(web_app_context)
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

enum WebServiceResult {
    Success(String),
    BadRequest(String),
    InternalError(String),
}

impl From<WebServiceResult> for (StatusCode, String) {
    fn from(value: WebServiceResult) -> Self {
        match value {
            WebServiceResult::Success(x) => (StatusCode::OK, x),
            WebServiceResult::BadRequest(x) => (StatusCode::BAD_REQUEST, x),
            WebServiceResult::InternalError(x) => (StatusCode::INTERNAL_SERVER_ERROR, x),
        }
    }
}

async fn create_iccp_association(State(state): State<WebAppContext>, Json(payload): Json<IccpAssociation>) -> (StatusCode, String) {
    let association: IccpAssociation = match payload.try_into() {
        Ok(association) => association,
        Err(_) => return WebServiceResult::InternalError("An error occurred that should not be possible.".into()).into(),
    };
    return try_create_iccp_association(state, association).await.into();
}

async fn try_create_iccp_association(state: WebAppContext, mut association: IccpAssociation) -> WebServiceResult {
    let mut config = state.config.write().await;
    if let Some(_) = config.iccp.associations.iter().find(|x| x.name == association.name) {
        return WebServiceResult::BadRequest("An association with the given name already exists".into());
    }

    let id = loop {
        let id = Uuid::new_v4().to_string();
        if let None = config.iccp.associations.iter().find(|x| x.id == id) {
            break id;
        }
    };
    association.id = id.clone();
    config.iccp.associations.push(association.clone());

    if let Err(e) = config.save().await {
        error!("{e}");
        return WebServiceResult::InternalError("Failed to store association".into());
    };
    drop(config);

    state.iccp_subsystem.write().await.create_association(association).await;
    WebServiceResult::Success(id)
}

async fn fetch_iccp_associations(State(context): State<WebAppContext>) -> (StatusCode, Json<Vec<IccpAssociationState>>) {
    (StatusCode::OK, Json(context.iccp_subsystem.read().await.list_associations().await))
}

async fn ws_handler(ws: WebSocketUpgrade, user_agent: Option<TypedHeader<headers::UserAgent>>, ConnectInfo(addr): ConnectInfo<SocketAddr>, State(context): State<WebAppContext>) -> impl IntoResponse {
    // Finalize the upgrade handshake
    ws.on_upgrade(move |socket| handle_socket(socket, context))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IccpAssociationStateMessage {
    pub kind: String,
    pub data: IccpAssociationState,
}

async fn handle_socket(mut socket: WebSocket, context: WebAppContext) {
    let (listener_sender, mut listener_receiver) = unbounded_channel();
    if let Err(_) = context.stream_queue_sender.send(listener_sender) {
        warn!("Cannot subscribe to websocket listener as it has already been closed.");
        return;
    }

    loop {
        tokio::select! {
            _ = socket.recv() => match socket.recv().await {
                Some(Ok(message)) => {
                    warn!("{message:?}");
                },
                Some(Err(e)) => {
                    error!("Failed to read from websocket: {e}");
                    return;
                },
                None => return,
            },
            message = listener_receiver.recv() => match message {
                Some(event) => {
                    match event {
                        IccpAssociationUpdate(iccp_association_state) => {
                            warn!("{iccp_association_state:?}");
                            let data = IccpAssociationStateMessage {
                                kind: "IccpAssociationStateMessage".into(),
                                data: iccp_association_state,
                            };
                            socket.send(Message::Text(serde_json::to_string_pretty(&data).unwrap().into())).await.unwrap()
                        },
                        IccpDataPointUpdate(_iccp_data_point_value) => (),
                    }
                },
                None => return,
            }
        }
    }
}
