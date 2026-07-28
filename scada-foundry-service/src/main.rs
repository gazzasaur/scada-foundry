pub mod config;
mod context;
pub mod core;
pub mod error;
pub mod iccp;

use std::net::SocketAddr;

use anyhow::Error;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{
        ConnectInfo,
        ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, post},
};
use axum_extra::{TypedHeader, headers};
use clap::Parser;
use futures::{SinkExt, StreamExt};
use oid::ObjectIdentifier;
use serde::{Deserialize, Serialize};
use tokio::join;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use uuid::Uuid;

use crate::{
    config::{ApplicationConfiguration, iccp::{
        AeTitle, AeTitleMatcher, IccpDataPoint, IccpDataSet, IccpInitiatorControlCenterInformation, IccpPointDataType, IccpPointName, IccpResponderControlCenterInformation, InitiatorAuthenticationScheme, InitiatorIccpAssociation,
        RemoteIccpControlCenterMatcher, ResponderIccpAssociation, SapAddressMatcher,
    }}, iccp::IccpManager,
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

    let iccp_manager = IccpManager::new().await;

    let app = Router::new().route("/", get(root)).route("/users", post(create_user)).route("/app/ws", any(ws_handler)).layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let y = iccp_manager.serve();
    let x = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>());

    let a = iccp_manager.clone();
    // tokio::task::spawn(async move {
    //     boo(a).await;
    // });
    let a = iccp_manager.clone();
    tokio::task::spawn(async move {
        yeah(a).await;
    });

    let (_, _) = join!(x, y);

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn boo(a: IccpManager) {
    a.initiator_iccp_association(InitiatorIccpAssociation {
        uuid: Uuid::new_v4().into(),
        name: "EGX_TO_GAZ".into(),
        role: config::iccp::InitiatorRole::Client,
        authentication: InitiatorAuthenticationScheme::None,
        local_control_center: IccpInitiatorControlCenterInformation {
            tsap_address: vec![1],
            ssap_address: vec![1],
            psap_address: vec![1],
            ae_title: AeTitle { ap_title: ObjectIdentifier::try_from("0.1.2.3.1").map_err(|e| anyhow::anyhow!("{:?}", e)).expect(""), ae_qualifier: 1.into() },
        },
        remote_control_center: IccpResponderControlCenterInformation {
            host: "127.0.0.1".into(),
            port: 8102,
            tsap_address: vec![2],
            ssap_address: vec![2],
            psap_address: vec![2],
            ae_title: AeTitle { ap_title: ObjectIdentifier::try_from("0.1.2.3.2").map_err(|e| anyhow::anyhow!("{:?}", e)).expect(""), ae_qualifier: 1.into() },
        },
        data_sets: vec![IccpDataSet {
            domain: "MyHouse".into(),
            name: "MyDataSet".into(),
            points: vec![IccpDataPoint { uuid: Uuid::new_v4().into(), name: IccpPointName::Icc("MyDataSet".into(), "MyPoint".into()), data_type: IccpPointDataType::State }],
        }],
    })
    .await
    .expect("");
}

async fn yeah(a: IccpManager) {
    let b = a
        .responder_iccp_association(ResponderIccpAssociation {
            uuid: Uuid::new_v4().into(),
            name: "EGX_TO_GAZ".into(),
            role: config::iccp::ResponderRole::Server,
            authentication: config::iccp::ResponderAuthenticationScheme::None,
            local_matcher: config::iccp::LocalIccpControlCenterMatcher::Masqurade,
            remote_matcher: RemoteIccpControlCenterMatcher::Relaxed {
                tsap_address: SapAddressMatcher::Any,
                ssap_address: SapAddressMatcher::Any,
                psap_address: SapAddressMatcher::Any,
                ae_title: AeTitleMatcher::ApTitleOnly(ObjectIdentifier::try_from(String::from("1.2.3.4")).expect("")),
            },
            points: vec![],
        })
        .await
        .expect("");
}

// async fn create_iccp_association(Json(payload): Json<CreateIccpAssociation>) -> (StatusCode, Json<String>) {

// }

async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
    let user = User { id: 1337, username: payload.username };
    (StatusCode::CREATED, Json(user))
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

// async fn ws_handler(
//     ws: WebSocketUpgrade,
//     user_agent: Option<TypedHeader<headers::UserAgent>>,
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
// ) -> impl IntoResponse {
//     println!("HELLO");
//     let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
//         user_agent.to_string()
//     } else {
//         String::from("Unknown browser")
//     };
//     println!("`{user_agent}` at {addr:?} connected.");
//     // finalize the upgrade process by returning upgrade callback.
//     // we can customize the callback by sending additional info such as address.
//     ws.on_upgrade(move |socket| handle_socket(socket, addr))
// }

// async fn ws_handler(
//     ws: WebSocketUpgrade,
//     user_agent: Option<TypedHeader<headers::UserAgent>>,
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
// ) -> impl IntoResponse {
//     let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
//         user_agent.to_string()
//     } else {
//         String::from("Unknown browser")
//     };
//     println!("`{user_agent}` at {addr} connected.");
//     // finalize the upgrade process by returning upgrade callback.
//     // we can customize the callback by sending additional info such as address.
//     ws.on_upgrade(move |socket| handle_socket(socket, addr))
// }

/// Actual websocket statemachine (one will be spawned per connection)
// async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
//     // send a ping (unsupported by some browsers) just to kick things off and get a response
//     if socket
//         .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
//         .await
//         .is_ok()
//     {
//         println!("Pinged {who:?}...");
//     } else {
//         println!("Could not send ping {who:?}!");
//         // no Error here since the only thing we can do is to close the connection.
//         // If we can not send messages, there is no way to salvage the statemachine anyway.
//         return;
//     }

//     // receive single message from a client (we can either receive or send with socket).
//     // this will likely be the Pong for our Ping or a hello message from client.
//     // waiting for message from a client will block this task, but will not block other client's
//     // connections.
//     if let Some(msg) = socket.recv().await {
//         if let Ok(msg) = msg {
//             // if process_message(msg, who).is_break() {
//             //     return;
//             // }
//         } else {
//             println!("client {who:?} abruptly disconnected");
//             return;
//         }
//     }

//     // Since each client gets individual statemachine, we can pause handling
//     // when necessary to wait for some external event (in this case illustrated by sleeping).
//     // Waiting for this client to finish getting its greetings does not prevent other clients from
//     // connecting to server and receiving their greetings.
//     for i in 1..5 {
//         if socket
//             .send(Message::Text(format!("Hi {i} times!").into()))
//             .await
//             .is_err()
//         {
//             println!("client {who:?} abruptly disconnected");
//             return;
//         }
//         tokio::time::sleep(std::time::Duration::from_millis(100)).await;
//     }

//     // By splitting socket we can send and receive at the same time. In this example we will send
//     // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
//     let (mut sender, mut receiver) = socket.split();

//     // Spawn a task that will push several messages to the client (does not matter what client does)
//     let whoc = who.clone();
//     let mut send_task = tokio::spawn(async move {
//         let n_msg = 20;
//         for i in 0..n_msg {
//             // In case of any websocket error, we exit.
//             if sender
//                 .send(Message::Text(format!("Server message {i} ...").into()))
//                 .await
//                 .is_err()
//             {
//                 return i;
//             }

//             tokio::time::sleep(std::time::Duration::from_millis(300)).await;
//         }

//         println!("Sending close to {whoc:?}...");
//         if let Err(e) = sender
//             .send(Message::Close(Some(CloseFrame {
//                 code: axum::extract::ws::close_code::NORMAL,
//                 reason: Utf8Bytes::from_static("Goodbye"),
//             })))
//             .await
//         {
//             println!("Could not send Close due to {e}, probably it is ok?");
//         }
//         n_msg
//     });

//     // This second task will receive messages from client and print them on server console
//     let mut recv_task = tokio::spawn(async move {
//         let mut cnt = 0;
//         while let Some(Ok(msg)) = receiver.next().await {
//             cnt += 1;
//             // print message and break if instructed to do so
//             // if process_message(msg, who).is_break() {
//             //     break;
//             // }
//         }
//         cnt
//     });

//     // If any one of the tasks exit, abort the other.
//     tokio::select! {
//         rv_a = (&mut send_task) => {
//             match rv_a {
//                 Ok(a) => println!("{a} messages sent to {who:?}"),
//                 Err(a) => println!("Error sending messages {a:?}")
//             }
//             recv_task.abort();
//         },
//         rv_b = (&mut recv_task) => {
//             match rv_b {
//                 Ok(b) => println!("Received {b} messages"),
//                 Err(b) => println!("Error receiving messages {b:?}")
//             }
//             send_task.abort();
//         }
//     }

//     // returning from the handler closes the websocket connection
//     println!("Websocket context {who:?} destroyed");
// }

//     ws: WebSocketUpgrade,
//     user_agent: Option<TypedHeader<headers::UserAgent>>,
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,

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
