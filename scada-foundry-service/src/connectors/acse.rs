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
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{
        RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    },
};
use tracing::{error, warn};

use crate::{api::ScadaFoundryEvent, config::acse::AcseIdentityParameters};

pub type EventQueue = UnboundedSender<ScadaFoundryEvent>;
pub type ConnectionQueue = UnboundedSender<Box<dyn RustyMmsServiceServer>>;

type RegistrationsMap = Arc<RwLock<HashMap<String, AcseListenerRegistration>>>;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcseListenerOperationalReport {
    pub id: String,
    pub name: String,

    pub host: String,
    pub port: u16,

    pub status: String,
    // pub status: String, TODO
    pub registrations: Vec<AcseListenerRegistrationOperationalReport>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcseListenerRegistrationOperationalReport {
    pub id: String,
    pub name: String,
    pub subsystem_name: String,
    pub called: AcseIdentityParameters,
    pub calling: AcseIdentityParameters,
}

pub struct AcseListenerRegistration {
    pub id: String,
    pub name: String,
    pub subsystem_name: String,
    pub listener: ConnectionQueue,

    pub called: AcseIdentityParameters,
    pub calling: AcseIdentityParameters,
}

pub struct AcseSubsystem {
    event_listener: EventQueue,
    listeners: Arc<RwLock<HashMap<String, AcseListenerManager>>>,
}

impl AcseSubsystem {
    pub fn new(event_listener: EventQueue) -> Self {
        return Self { listeners: Arc::new(RwLock::new(HashMap::new())), event_listener };
    }

    pub async fn list_listeners(&self) -> Vec<AcseListenerOperationalReport> {
        let mut reports = vec![];
        for listener in self.listeners.read().await.values() {
            let mut registrations = vec![];
            for registration in listener.registrations.read().await.values() {
                registrations.push(AcseListenerRegistrationOperationalReport {
                    id: registration.id.clone(),
                    name: registration.name.clone(),
                    subsystem_name: registration.subsystem_name.clone(),
                    called: registration.called.clone(),
                    calling: registration.calling.clone(),
                });
            }
            reports.push(AcseListenerOperationalReport { id: listener.id.clone(), name: listener.name.clone(), host: listener.host.clone(), port: listener.port, status: listener.status.read().await.clone(), registrations: vec![] });
        }
        reports
    }

    pub async fn listen(&mut self, id: String, name: String, host: String, port: u16) -> AcseListenerOperationalReport {
        let mut listeners = self.listeners.write().await;

        return match listeners.entry(id.clone()) {
            Entry::Occupied(mut occupied_entry) => {
                let _ = occupied_entry.get()._notifier.send(());
                occupied_entry.get()._notifier.closed().await;

                let listener_record = AcseListenerOperationalReport { id: id.clone(), name: name.clone(), host: host.clone(), port, status: "Initialising".into(), registrations: vec![] };

                let registrations = occupied_entry.get_mut().registrations.write().await.drain().collect::<Vec<_>>();
                let mut new_entry = AcseListenerManager::new(id.clone(), name.clone(), host.clone(), port, self.event_listener.clone());
                for (_, registration) in registrations {
                    new_entry.register(registration).await;
                }
                occupied_entry.insert(new_entry);
                listener_record
            }
            Entry::Vacant(vacant_entry) => {
                let listener_record = AcseListenerOperationalReport { id: id.clone(), name: name.clone(), host: host.clone(), port, status: "Initialising".into(), registrations: vec![] };
                vacant_entry.insert_entry(AcseListenerManager::new(id, name, host, port, self.event_listener.clone()));
                listener_record
            }
        };
    }

    pub async fn close(&mut self, id: String) {
        let mut listeners = self.listeners.write().await;

        return match listeners.entry(id.clone()) {
            Entry::Occupied(occupied_entry) => {
                let _ = occupied_entry.get()._notifier.send(());
                occupied_entry.get()._notifier.closed().await;
                let item = occupied_entry.remove();
                let _ = self.event_listener.send(ScadaFoundryEvent::AcseListenerReport(AcseListenerOperationalReport { id: item.id, name: item.name, host: item.host, port: item.port, status: "Deleted".into(), registrations: vec![] }));
            }
            Entry::Vacant(_) => (),
        };
    }

    pub async fn register(&mut self, id: String, registration: AcseListenerRegistration) {
        match self.listeners.write().await.entry(id.clone()) {
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

    id: String,
    name: String,

    host: String,
    port: u16,

    status: Arc<RwLock<String>>,
    registrations: RegistrationsMap,
}

impl AcseListenerManager {
    pub fn new(id: String, name: String, host: String, port: u16, event_listener: EventQueue) -> Self {
        let status = Arc::new(RwLock::new("Idle".into()));
        let registrations = Arc::new(RwLock::new(HashMap::new()));
        let (notifier_sender, notifier_receiver) = unbounded_channel();
        tokio::task::spawn(acse_worker(id.clone(), name.clone(), status.clone(), host.clone(), port, registrations.clone(), event_listener, notifier_receiver));
        Self { id, name, host, port, status, _notifier: notifier_sender, registrations }
    }

    pub async fn state(&self) -> String {
        (*&self.status.read().await).clone()
    }

    pub async fn register(&mut self, registration: AcseListenerRegistration) {
        self.registrations.write().await.insert(registration.id.clone(), registration);
    }
}

async fn acse_worker(id: String, name: String, state: Arc<RwLock<String>>, host: String, port: u16, registrations: RegistrationsMap, mut event_listener: EventQueue, mut notifier: UnboundedReceiver<()>) {
    loop {
        *state.write().await = "Idle".into();
        // TODO Registrations
        match event_listener.send(ScadaFoundryEvent::AcseListenerReport(AcseListenerOperationalReport { id: id.clone(), name: name.clone(), host: host.clone(), port: port, status: "Idle".into(), registrations: vec![] })) {
            Ok(()) => (),
            Err(_) => return,
        }

        match try_acse_worker(id.clone(), name.clone(), state.clone(), &host, port, &registrations, &mut event_listener, &mut notifier).await {
            Ok(()) => return,
            Err(e) => {
                error!("{e}");
                *state.write().await = "Error".into();
                // TODO Registrations
                match event_listener.send(ScadaFoundryEvent::AcseListenerReport(AcseListenerOperationalReport { id: id.clone(), name: name.clone(), host: host.clone(), port, status: "Error".into(), registrations: vec![] })) {
                    Ok(()) => (),
                    Err(_) => return,
                }
            }
        }
        select! {
            _ = tokio::time::sleep(Duration::from_secs(15 + random_range(0u64..15u64))) => (),
            _ = notifier.recv() => return,
        }
    }
}

async fn try_acse_worker(id: String, name: String, state: Arc<RwLock<String>>, host: &String, port: u16, registrations: &RegistrationsMap, event_listener: &mut EventQueue, notifier: &mut UnboundedReceiver<()>) -> Result<(), anyhow::Error> {
    *state.write().await = "Opening".into();
    // TODO registrations
    match event_listener.send(ScadaFoundryEvent::AcseListenerReport(AcseListenerOperationalReport { id: id.clone(), name: name.clone(), host: host.clone(), port, status: "Opening".into(), registrations: vec![] })) {
        Ok(()) => (),
        Err(_) => return Ok(()),
    }

    let address = format!("{host}:{port}").parse()?;
    let listener = TcpTpktServer::listen(address).await?;

    *state.write().await = "Listening".into();
    // TODO Registrations
    match event_listener.send(ScadaFoundryEvent::AcseListenerReport(AcseListenerOperationalReport { id: id.clone(), name: name.clone(), host: host.clone(), port, status: "Listening".into(), registrations: vec![] })) {
        Ok(()) => (),
        Err(_) => return Ok(()),
    }

    loop {
        tokio::select! {
            connection = listener.accept() => tokio::task::spawn(acse_connection_worker(connection?, registrations.clone())),
            _ = notifier.recv() => return Ok(())
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
            && registration.calling.tsap_id.as_ref() == cotp_info.calling_tsap_id()
            && registration.calling.session_selector.as_ref() == cosp_info.calling_session_selector()
            && registration.calling.presentation_selector.as_ref() == copp_info.calling_presentation_selector.as_ref()
            && &registration.calling.ae_title.ap_title == calling_ap_title
            && registration.calling.ae_title.ae_qualifier == BigInt::from_signed_bytes_be(&calling_ae_qualifier)
            && registration.called.tsap_id.as_ref() == cotp_info.called_tsap_id()
            && registration.called.session_selector.as_ref() == cosp_info.called_session_selector()
            && registration.called.presentation_selector.as_ref() == copp_info.called_presentation_selector.as_ref()
            && &registration.called.ae_title.ap_title == called_ap_title
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
