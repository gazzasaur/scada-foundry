pub mod config;
pub mod error;
pub mod iccp;

use anyhow::Error;
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use clap::Parser;
use tokio::join;

use crate::{config::ApplicationConfiguration, iccp::IccpManager};

/// SCADA Foundry Server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The file containing the server configuration
    #[arg(short, long, default_value = "/home/gaz/scada-foundry/scada-foundry-service/config.json")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    // let args = Args::parse();
    // let app_config = ApplicationConfiguration::load(args.config_file.as_str()).await?;

    let iccp_manager = IccpManager::new().await;

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let y = iccp_manager.serve();
    let x= axum::serve(listener, app);

    let (_, _) = join!(x, y);

    // // TODO Serialise JSON in another thread
    // let a = IccpConfiguration {
    //     initator_associations: vec![InitiatorIccpAssociation {
    //         uuid: Uuid::new_v4().into(),
    //         name: "EGX_TO_GAZ".into(),
    //         role: config::iccp::InitiatorRole::Client,
    //         authentication: InitiatorAuthenticationScheme::None,
    //         local_control_center: IccpInitiatorControlCenterInformation {
    //             tsap_address: vec![1],
    //             ssap_address: vec![1],
    //             psap_address: vec![1],
    //             ae_title: AeTitle {
    //                 ap_title: ObjectIdentifier::try_from("0.1.2.3.1")
    //                     .map_err(|e| anyhow::anyhow!("{:?}", e))?,
    //                 ae_qualifier: 1.into(),
    //             },
    //         },
    //         remote_control_center: IccpResponderControlCenterInformation {
    //             host: "127.0.0.1".into(),
    //             port: 10002,
    //             tsap_address: vec![2],
    //             ssap_address: vec![2],
    //             psap_address: vec![2],
    //             ae_title: AeTitle {
    //                 ap_title: ObjectIdentifier::try_from("0.1.2.3.2")
    //                     .map_err(|e| anyhow::anyhow!("{:?}", e))?,
    //                 ae_qualifier: 1.into(),
    //             },
    //         },
    //         data_sets: vec![IccpDataSet {
    //             domain: "MyHouse".into(),
    //             name: "MyDataSet".into(),
    //             points: vec![IccpDataPoint {
    //                 uuid: Uuid::new_v4().into(),
    //                 name: IccpPointName::Icc("MyDataSet".into(), "MyPoint".into()),
    //                 data_type: IccpPointDataType::State,
    //             }],
    //         }],
    //     }],
    //     responder_associations: vec![ResponderIccpAssociation {
    //         uuid: Uuid::new_v4().into(),
    //         name: "GAZ_TO_EGX".into(),
    //         role: ResponderRole::Server,
    //         authentication: ResponderAuthenticationScheme::None,
    //         local_matcher: LocalIccpControlCenterMatcher::Masqurade,
    //         remote_matcher: RemoteIccpControlCenterMatcher::Relaxed {
    //             tsap_address: SapAddressMatcher::Any,
    //             ssap_address: SapAddressMatcher::Any,
    //             psap_address: SapAddressMatcher::Any,
    //             ae_title: AeTitleMatcher::ApTitleOnly(
    //                 ObjectIdentifier::try_from("0.1.2.3.1").map_err(|e| anyhow::anyhow!("{:?}", e))?,
    //             ),
    //         },
    //         points: vec![IccpDataPoint {
    //             uuid: Uuid::new_v4().into(),
    //             name: IccpPointName::Icc("MyDataSet".into(), "MyPoint".into()),
    //             data_type: IccpPointDataType::State,
    //         }],
    //     }],
    // };
    // let app_config = ApplicationConfiguration { iccp: a };
    // let json_data = serde_json::to_vec_pretty(&app_config)?;

    // let mut file = File::create("config.json").await?;
    // file.write_all(&json_data).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

// async fn create_iccp_association(Json(payload): Json<CreateIccpAssociation>) -> (StatusCode, Json<String>) {

// }

async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1337,
        username: payload.username,
    };
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
