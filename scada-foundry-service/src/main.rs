pub mod config;
pub mod error;
pub mod iccp;
pub mod core;

use anyhow::Error;
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use clap::Parser;
use oid::ObjectIdentifier;
use serde::{Deserialize, Serialize};
use tokio::join;
use uuid::Uuid;

use crate::{
    config::iccp::{
        AeTitle, AeTitleMatcher, IccpDataPoint, IccpDataSet, IccpInitiatorControlCenterInformation, IccpPointDataType, IccpPointName, IccpResponderControlCenterInformation, InitiatorAuthenticationScheme, InitiatorIccpAssociation,
        RemoteIccpControlCenterMatcher, ResponderIccpAssociation, SapAddressMatcher,
    },
    iccp::IccpManager,
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

    // let args = Args::parse();
    // let app_config = ApplicationConfiguration::load(args.config_file.as_str()).await?;

    let iccp_manager = IccpManager::new().await;

    let app = Router::new().route("/", get(root)).route("/users", post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let y = iccp_manager.serve();
    let x = axum::serve(listener, app);

    let a = iccp_manager.clone();
    tokio::task::spawn(async move {
        boo(a).await;
    });
    let a = iccp_manager.clone();
    tokio::task::spawn(async move {
        yeah(a).await;
    });
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
    a.responder_iccp_association(ResponderIccpAssociation {
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
