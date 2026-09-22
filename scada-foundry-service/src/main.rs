pub mod api;
pub mod config;
pub mod connectors;
mod context;
pub mod core;
pub mod error;
pub mod iccp;
mod model;
pub mod webservice;

use std::{
    collections::{HashMap, VecDeque, hash_map::Entry},
    net::SocketAddr,
    sync::Arc,
};

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
use serde::{Deserialize, Serialize};
use tokio::{
    join,
    sync::{
        RwLock,
        mpsc::{UnboundedSender, unbounded_channel},
    },
};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{error, warn};
use uuid::{Uuid, uuid};

use crate::{
    api::ScadaFoundryEvent::{self, AcseListenerReport, IccpDataPointUpdate}, config::{ApplicationConfiguration, acse::AcseListener}, connectors::{acse::{AcseListenerOperationalReport, AcseSubsystem}, iccp::{IccpServerAssocaitionOperationalReport, IccpSubsystem}}, iccp::api::{IccpAssociation, IccpServerOperationalAssociation},
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

    pub acse_subsystem: Arc<RwLock<AcseSubsystem>>,
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

    let acse_subsystem = Arc::new(RwLock::new(AcseSubsystem::new(global_listener_sender.clone())));
    let iccp_subsystem = Arc::new(RwLock::new(IccpSubsystem::new(global_listener_sender.clone())));
    let web_app_context = WebAppContext { acse_subsystem: acse_subsystem.clone(), iccp_subsystem: iccp_subsystem.clone(), config: app_config.clone(), stream_queue_sender };

    {
        let app_config = app_config.read().await;
        let mut acse_subsystem = acse_subsystem.write().await;

        for acse_listener in app_config.acse.acse_listeners.iter() {
            match acse_listener {
                AcseListener { id, name, host, port } => acse_subsystem.listen(id.clone(), name.clone(), host.clone(), *port).await,
            };
        }
    }

    // TODO This lazily removes queues when events occur.
    // Can also call 'closed' on all the queues to remove them immediately after they are closed.
    let queue_worker_iccp_manager = iccp_subsystem.clone();
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
        // iccp_subsystem.write().await.create_association(iccp_association).await;
    }

    let cors = CorsLayer::permissive();
    let app = Router::new()
        .route("/", get(root))
        .route("/app/api/fetchacselisteners", get(fetch_acse_listeners))
        .route("/app/api/createacselistener", post(create_acse_listener))
        .route("/app/api/updateacselistener", post(update_acse_listener))
        .route("/app/api/deleteacselistener", post(delete_acse_listener))
        .route("/app/api/fetchiccpassociations", get(fetch_iccp_associations))
        .route("/app/api/createiccpassociation", post(create_iccp_association))
        // .route("/app/api/fetchiccpdatapoints", get(fetch_iccp_data_points))
        // .route("/app/api/createiccpdatapoint", post(create_iccp_data_point))
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

enum WebServiceResult<T> {
    Success(T),
    BadRequest,
    InternalError,
}

impl<T> WebServiceResult<T> {
    pub fn success(value: T) -> Self {
        WebServiceResult::Success(value)
    }

    pub fn bad_reqeust(message: String) -> Self {
        error!(message);
        WebServiceResult::BadRequest
    }

    pub fn internal_error(message: String) -> Self {
        error!(message);
        WebServiceResult::InternalError
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityIdentifier {
    pub id: String,
}

impl<T> From<WebServiceResult<T>> for Result<T, StatusCode> {
    fn from(value: WebServiceResult<T>) -> Self {
        match value {
            WebServiceResult::Success(x) => Ok(x),
            WebServiceResult::BadRequest => Err(StatusCode::BAD_REQUEST),
            WebServiceResult::InternalError => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

async fn fetch_acse_listeners(State(context): State<WebAppContext>) -> Result<Json<Vec<AcseListenerOperationalReport>>, StatusCode> {
    Ok(Json(context.acse_subsystem.read().await.list_listeners().await))
}

async fn create_acse_listener(State(context): State<WebAppContext>, Json(payload): Json<AcseListener>) -> Result<Json<AcseListenerOperationalReport>, StatusCode> {
    let listener: AcseListener = match payload.try_into() {
        Ok(x) => x,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    return try_create_acse_listener(context, listener).await.into();
}

async fn try_create_acse_listener(context: WebAppContext, listener: AcseListener) -> WebServiceResult<Json<AcseListenerOperationalReport>> {
    let mut config = context.config.write().await;

    let id = loop {
        let id = Uuid::new_v4().to_string();
        if let None = config.acse.acse_listeners.iter().find(|x| x.id == id) {
            break id;
        }
    };

    config.acse.acse_listeners.push(AcseListener { id: id.clone(), name: listener.name.clone(), host: listener.host.clone(), port: listener.port });
    if let Err(e) = config.save().await {
        return WebServiceResult::internal_error(format!("{e:?}"));
    }

    // Grab the next lock before freeing the previous one.
    let mut subsystem = context.acse_subsystem.write().await;
    drop(config);

    let record = subsystem.listen(id, listener.name, listener.host, listener.port).await;
    WebServiceResult::Success(Json(record))
}

async fn update_acse_listener(State(context): State<WebAppContext>, Json(payload): Json<AcseListener>) -> Result<Json<AcseListenerOperationalReport>, StatusCode> {
    let listener: AcseListener = match payload.try_into() {
        Ok(x) => x,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    return try_update_acse_listener(context, listener).await.into();
}

async fn try_update_acse_listener(context: WebAppContext, listener: AcseListener) -> WebServiceResult<Json<AcseListenerOperationalReport>> {
    let mut config = context.config.write().await;

    config.acse.acse_listeners.as_mut_slice().iter_mut().find(|x| x.id == listener.id).iter_mut().for_each(|x| {
        x.name = listener.name.clone();
        x.host = listener.host.clone();
        x.port = listener.port;
    });
    if let Err(e) = config.save().await {
        return WebServiceResult::internal_error(format!("{e:?}"));
    }

    // Grab the next lock before freeing the previous one.
    let mut subsystem = context.acse_subsystem.write().await;
    drop(config);

    let record = subsystem.listen(listener.id, listener.name, listener.host, listener.port).await;
    WebServiceResult::Success(Json(record))
}

async fn delete_acse_listener(State(context): State<WebAppContext>, Json(payload): Json<EntityIdentifier>) -> Result<(), StatusCode> {
    let listener: EntityIdentifier = match payload.try_into() {
        Ok(x) => x,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    try_delete_acse_listener(context, listener.id).await
}

async fn try_delete_acse_listener(context: WebAppContext, listener_id: String) -> Result<(), StatusCode> {
    let mut config = context.config.write().await;

    config.acse.acse_listeners = config.acse.acse_listeners.drain(..).filter(|x| x.id != listener_id).collect();
    if let Err(e) = config.save().await {
        return WebServiceResult::internal_error(format!("{e:?}")).into();
    }

    // Grab the next lock before freeing the previous one.
    let mut subsystem = context.acse_subsystem.write().await;
    drop(config);

    subsystem.close(listener_id).await;
    Ok(())
}

async fn fetch_iccp_associations(State(context): State<WebAppContext>) -> (StatusCode, Json<Vec<IccpServerAssocaitionOperationalReport>>) {
    (StatusCode::OK, Json(context.iccp_subsystem.read().await.list_server_associations().await))
}

async fn create_iccp_association(State(state): State<WebAppContext>, Json(payload): Json<IccpAssociation>) -> Result<Json<String>, StatusCode> {
    let association: IccpAssociation = match payload.try_into() {
        Ok(association) => association,
        Err(_) => return WebServiceResult::internal_error("An error occurred that should not be possible.".into()).into(),
    };
    return try_create_iccp_association(state, association).await.into();
}

async fn try_create_iccp_association(state: WebAppContext, mut association: IccpAssociation) -> WebServiceResult<Json<String>> {
    let mut config = state.config.write().await;
    if let Some(_) = config.iccp.associations.iter().find(|x| x.name == association.name) {
        return WebServiceResult::bad_reqeust("An association with the given name already exists".into());
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
        return WebServiceResult::internal_error("Failed to store association".into());
    };
    drop(config);

    state.iccp_subsystem.write().await.create_server_association(id.clone(), association.name, association.remote_data_center_parameters, association.local_data_center_parameters).await;
    WebServiceResult::success(Json(id))
}

// async fn fetch_iccp_data_points(State(context): State<WebAppContext>) -> (StatusCode, Json<Vec<IccpDataPointValue>>) {
//     (StatusCode::OK, Json(context.iccp_subsystem.read().await.list_data_points().await))
// }

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebServiceIccpDataPointSpecification {
    pub association_id: String,
    pub data_point_name: String,
    pub data_point_type: String,
    pub data_point_domain: Option<String>,
}

async fn ws_handler(ws: WebSocketUpgrade, user_agent: Option<TypedHeader<headers::UserAgent>>, ConnectInfo(addr): ConnectInfo<SocketAddr>, State(context): State<WebAppContext>) -> impl IntoResponse {
    // Finalize the upgrade handshake
    ws.on_upgrade(move |socket| handle_socket(socket, context))
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcseListenerReportMessage {
    pub kind: String,
    pub data: AcseListenerOperationalReport,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IccpAssociationStateMessage {
    pub kind: String,
    pub data: IccpServerOperationalAssociation,
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
                        AcseListenerReport(acse_connector_status) => {
                            let data = AcseListenerReportMessage {
                                kind: "AcseListenerReportMessage".into(),
                                data: acse_connector_status,
                            };
                            socket.send(Message::Text(serde_json::to_string_pretty(&data).unwrap().into())).await.unwrap()
                        },

                        IccpDataPointUpdate(_iccp_data_point_value) => (),
                        ScadaFoundryEvent::IccpServerAssociationReport(iccp_assocaition_operational_report) => todo!(),
                    }
                },
                None => return,
            }
        }
    }
}
