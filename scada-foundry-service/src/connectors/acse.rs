use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
    time::Duration,
};

use num_bigint::BigInt;
use rand::random_range;
use rusty_acse::{AeQualifier, ApTitle, RustyOsiSingleValueAcseListenerIsoStack};
use rusty_copp::RustyCoppListenerIsoStack;
use rusty_cosp::{CospConnectionParameters, RustyCospAcceptorIsoStack};
use rusty_cotp::{CotpConnectionParameters, CotpResponder, RustyCotpResponder};
use rusty_mms_service::{RustyMmsServiceServer, accept_mms_service_server_connect};
use rusty_tpkt::{TcpTpktConnection, TcpTpktReader, TcpTpktServer, TcpTpktWriter};
use tokio::sync::{
    RwLock,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tracing::{error, warn};

use crate::{
    api::ScadaFoundryEvent,
    iccp::{api::IccpDataCenterParameters, converter::convert_object_identifiers},
};

pub type EventQueue = UnboundedSender<ScadaFoundryEvent>;
pub type ConnectionQueue = UnboundedSender<Box<dyn RustyMmsServiceServer>>;

type RegistrationsMap = Arc<RwLock<HashMap<String, AcseListenerRegistration>>>;

#[derive(Clone, Debug)]
pub struct AcseConnectorStatus {
    pub id: String,
    pub state: String,
}

struct AcseListenerRegistration {
    pub id: String,
    pub listener: ConnectionQueue,

    pub called: IccpDataCenterParameters,
    pub calling: IccpDataCenterParameters,
}

pub struct AcseSubsystem {
    event_listener: EventQueue,
    managers: Arc<RwLock<HashMap<String, AcseListenerManager>>>,
}

impl AcseSubsystem {
    pub fn new(event_listener: EventQueue) -> Self {
        return Self { managers: Arc::new(RwLock::new(HashMap::new())), event_listener };
    }

    pub async fn listen(&mut self, id: String, host: String, port: u16) {
        self.managers.write().await.insert(id.clone(), AcseListenerManager::new(id, host, port, self.event_listener.clone()));
    }

    pub async fn register(&mut self, id: String, registration: AcseListenerRegistration) {
        match self.managers.write().await.entry(id.clone()) {
            Entry::Occupied(x) => {
                x.get().registrations.write().await.insert(registration.id.clone(), registration);
                ()
            }
            Entry::Vacant(_) => (), // Race condition, already deleted.
        };
    }
}

pub struct AcseListenerManager {
    // This will terminate the worked if the parent goes out of scope.
    _notifier: UnboundedSender<()>,

    state: Arc<RwLock<String>>,
    registrations: RegistrationsMap,
}

impl AcseListenerManager {
    pub fn new(id: String, host: String, port: u16, event_listener: EventQueue) -> Self {
        let state = Arc::new(RwLock::new("Idle".into()));
        let registrations = Arc::new(RwLock::new(HashMap::new()));
        let (notifier_sender, notifier_receiver) = unbounded_channel();
        tokio::task::spawn(acse_worker(id, state.clone(), host.clone(), port, registrations.clone(), event_listener, notifier_receiver));
        Self { state, _notifier: notifier_sender, registrations }
    }

    pub async fn state(&self) -> String {
        (*&self.state.read().await).clone()
    }

    pub async fn register(&mut self, registration: AcseListenerRegistration) {
        self.registrations.write().await.insert(registration.id.clone(), registration);
    }
}

async fn acse_worker(id: String, state: Arc<RwLock<String>>, host: String, port: u16, registrations: RegistrationsMap, mut event_listener: EventQueue, mut notifier: UnboundedReceiver<()>) {
    loop {
        *state.write().await = "Idle".into();
        match event_listener.send(ScadaFoundryEvent::AcseConnectorStatusUpdate(AcseConnectorStatus { id: id.clone(), state: "Idle".into() })) {
            Ok(()) => (),
            Err(_) => return,
        }

        match try_acse_worker(id.clone(), state.clone(), &host, port, &registrations, &mut event_listener, &mut notifier).await {
            Ok(()) => return,
            Err(e) => {
                error!("{e}");
                *state.write().await = "Error".into();
                match event_listener.send(ScadaFoundryEvent::AcseConnectorStatusUpdate(AcseConnectorStatus { id: id.clone(), state: "Error".into() })) {
                    Ok(()) => (),
                    Err(_) => return,
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(15 + random_range(0u64..15u64))).await;
    }
}

async fn try_acse_worker(id: String, state: Arc<RwLock<String>>, host: &String, port: u16, registrations: &RegistrationsMap, event_listener: &mut EventQueue, notifier: &mut UnboundedReceiver<()>) -> Result<(), anyhow::Error> {
    *state.write().await = "Opening".into();
    match event_listener.send(ScadaFoundryEvent::AcseConnectorStatusUpdate(AcseConnectorStatus { id: id.clone(), state: "Opening".into() })) {
        Ok(()) => (),
        Err(_) => return Ok(()),
    }

    let address = format!("{host}:{port}").parse()?;
    let listener = TcpTpktServer::listen(address).await?;

    *state.write().await = "Listening".into();
    match event_listener.send(ScadaFoundryEvent::AcseConnectorStatusUpdate(AcseConnectorStatus { id: id.clone(), state: "Listening".into() })) {
        Ok(()) => (),
        Err(_) => return Ok(()),
    }

    loop {
        tokio::select! {
            connection = listener.accept() => tokio::task::spawn(acse_connection_worker(connection?, registrations.clone())),
            _ = notifier.recv() => return Ok(()),
        };
        tokio::time::sleep(Duration::from_millis(10 + random_range(0u64..5u64))).await;
    }
}

async fn acse_connection_worker(connection: TcpTpktConnection, registrations: RegistrationsMap) {
    match tokio::time::timeout(Duration::from_secs(30), try_acse_connection_worker(connection, &registrations)).await {
        Ok(Ok(())) => (),
        Ok(Err(e)) => {
            error!("Failed to establish mms server connection: {e}")
        }
        Err(e) => {
            error!("Timeout waiting to establish mms server connection: {e}")
        }
    };
}

// TODO Host Check
async fn try_acse_connection_worker(connection: TcpTpktConnection, registrations: &RegistrationsMap) -> Result<(), anyhow::Error> {
    let (responder, cotp_info) = RustyCotpResponder::<TcpTpktReader, TcpTpktWriter>::new(connection, CotpConnectionParameters::default()).await?;
    let cotp_connection = responder.accept(cotp_info.clone().responder()).await?;
    let (cosp_acceptor, cosp_info) = RustyCospAcceptorIsoStack::<TcpTpktReader, TcpTpktWriter>::new(cotp_connection, CospConnectionParameters::default()).await?;
    let (copp_listener, copp_info) = RustyCoppListenerIsoStack::<TcpTpktReader, TcpTpktWriter>::new(cosp_acceptor).await?;
    let (acse_listener, acse_info) = RustyOsiSingleValueAcseListenerIsoStack::<TcpTpktReader, TcpTpktWriter>::new(copp_listener).await?;

    for registration in registrations.read().await.values() {
        if let Some(ApTitle::Form2(calling_ap_title)) = &acse_info.calling_ap_title
            && let Some(ApTitle::Form2(called_ap_title)) = &acse_info.called_ap_title
            && let Some(AeQualifier::Form2(calling_ae_qualifier)) = &acse_info.calling_ae_qualifier
            && let Some(AeQualifier::Form2(called_ae_qualifier)) = &acse_info.called_ae_qualifier
            && Some(&registration.calling.tsap) == cotp_info.calling_tsap_id()
            && Some(&registration.calling.ssap) == cosp_info.calling_session_selector()
            && Some(&registration.calling.psap) == copp_info.calling_presentation_selector.as_ref()
            && &convert_object_identifiers(&registration.calling.ae_title.ap_title)? == calling_ap_title
            && registration.calling.ae_title.ae_qualifier == BigInt::from_signed_bytes_be(&calling_ae_qualifier)
            && Some(&registration.called.tsap) == cotp_info.called_tsap_id()
            && Some(&registration.called.ssap) == cosp_info.called_session_selector()
            && Some(&registration.called.psap) == copp_info.called_presentation_selector.as_ref()
            && &convert_object_identifiers(&registration.called.ae_title.ap_title)? == called_ap_title
            && registration.called.ae_title.ae_qualifier == BigInt::from_signed_bytes_be(&called_ae_qualifier)
        {
            let connection = accept_mms_service_server_connect::<TcpTpktReader, TcpTpktWriter>(acse_listener).await?;
            if let Err(_) = registration.listener.send(connection) {
                warn!("A dead registration is still active. This can be normal if a connection is being processed at the same time it is removed, but this should be very rare.");
            }
            return Ok(());
        }
    }
    Ok(())
}
