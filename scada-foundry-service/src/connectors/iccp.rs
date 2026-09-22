use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
    time::Duration,
};

use rand::random_range;
use rusty_mms_service::RustyMmsServiceServer;
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{
        RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    },
};
use tracing::error;

use crate::{api::ScadaFoundryEvent, config::acse::AcseIdentityParameters};

pub type EventQueue = UnboundedSender<ScadaFoundryEvent>;
pub type ConnectionQueue = UnboundedSender<Box<dyn RustyMmsServiceServer>>;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IccpServerAssocaitionOperationalReport {
    pub id: String,
    pub name: String,
    pub acse_listener: Option<String>,

    pub calling: AcseIdentityParameters,
    pub called: AcseIdentityParameters,

    pub status: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IccpListenerRegistrationOperationalReport {
    pub id: String,
    pub name: String,
    pub subsystem_name: String,
    pub called: AcseIdentityParameters,
    pub calling: AcseIdentityParameters,
}

pub struct IccpListenerRegistration {
    pub id: String,
    pub name: String,
    pub subsystem_name: String,
    pub listener: ConnectionQueue,

    pub called: AcseIdentityParameters,
    pub calling: AcseIdentityParameters,
}

pub struct IccpSubsystem {
    event_listener: EventQueue,
    associations: Arc<RwLock<HashMap<String, IccpAssociationManager>>>,
}

impl IccpSubsystem {
    pub fn new(event_listener: EventQueue) -> Self {
        return Self { associations: Arc::new(RwLock::new(HashMap::new())), event_listener };
    }

    pub async fn list_server_associations(&self) -> Vec<IccpServerAssocaitionOperationalReport> {
        let mut reports = vec![];
        for association in self.associations.read().await.values() {
            reports.push(IccpServerAssocaitionOperationalReport {
                id: association.id.clone(),
                name: association.name.clone(),
                status: association.status.read().await.clone(),
                calling: association.calling.clone(),
                called: association.called.clone(),
                acse_listener: None,
            });
        }
        reports
    }

    pub async fn create_server_association(&mut self, id: String, name: String, calling: AcseIdentityParameters, called: AcseIdentityParameters) -> IccpServerAssocaitionOperationalReport {
        let mut associations = self.associations.write().await;

        return match associations.entry(id.clone()) {
            Entry::Occupied(mut occupied_entry) => {
                let _ = occupied_entry.get()._notifier.send(());
                occupied_entry.get()._notifier.closed().await;

                let association_record = IccpServerAssocaitionOperationalReport { id: id.clone(), name: name.clone(), acse_listener: None, status: "Initialising".into(), calling: calling.clone(), called: called.clone() };
                occupied_entry.insert(IccpAssociationManager::new(id.clone(), name.clone(), calling, called, self.event_listener.clone()));
                association_record
            }
            Entry::Vacant(vacant_entry) => {
                let listener_record = IccpServerAssocaitionOperationalReport { id: id.clone(), name: name.clone(), acse_listener: None, status: "Initialising".into(), calling: calling.clone(), called: called.clone() };
                vacant_entry.insert_entry(IccpAssociationManager::new(id, name, calling, called, self.event_listener.clone()));
                listener_record
            }
        };
    }

    pub async fn close(&mut self, id: String) {
        let mut assocaitions = self.associations.write().await;

        return match assocaitions.entry(id.clone()) {
            Entry::Occupied(occupied_entry) => {
                let _ = occupied_entry.get()._notifier.send(());
                occupied_entry.get()._notifier.closed().await;
                let item = occupied_entry.remove();
                let _ = self.event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssocaitionOperationalReport {
                    id: item.id,
                    name: item.name,
                    acse_listener: None,
                    status: "Deleted".into(),
                    calling: item.calling,
                    called: item.called,
                }));
            }
            Entry::Vacant(_) => (),
        };
    }
}

pub struct IccpAssociationManager {
    // This will terminate the worked if the parent goes out of scope.
    _notifier: UnboundedSender<()>,

    id: String,
    name: String,

    calling: AcseIdentityParameters,
    called: AcseIdentityParameters,

    status: Arc<RwLock<String>>,
}

impl IccpAssociationManager {
    pub fn new(id: String, name: String, calling: AcseIdentityParameters, called: AcseIdentityParameters, event_listener: EventQueue) -> Self {
        let status = Arc::new(RwLock::new("Idle".into()));
        let (notifier_sender, notifier_receiver) = unbounded_channel();
        tokio::task::spawn(iccp_worker(id.clone(), name.clone(), status.clone(), calling.clone(), called.clone(), event_listener, notifier_receiver));
        Self { id, name, status, calling, called, _notifier: notifier_sender }
    }

    pub async fn state(&self) -> String {
        (*&self.status.read().await).clone()
    }
}

async fn iccp_worker(id: String, name: String, state: Arc<RwLock<String>>, calling: AcseIdentityParameters, called: AcseIdentityParameters, mut event_listener: EventQueue, mut notifier: UnboundedReceiver<()>) {
    loop {
        *state.write().await = "Idle".into();
        // TODO Registrations
        match event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssocaitionOperationalReport {
            id: id.clone(),
            name: name.clone(),
            acse_listener: None,
            status: "Idle".into(),
            calling: calling.clone(),
            called: called.clone(),
        })) {
            Ok(()) => (),
            Err(_) => return,
        }

        match try_iccp_worker(id.clone(), name.clone(), state.clone(), calling.clone(), called.clone(), &mut event_listener, &mut notifier).await {
            Ok(()) => return,
            Err(e) => {
                error!("{e}");
                *state.write().await = "Error".into();
                // TODO Registrations
                match event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssocaitionOperationalReport {
                    id: id.clone(),
                    name: name.clone(),
                    acse_listener: None,
                    status: "Error".into(),
                    calling: calling.clone(),
                    called: called.clone(),
                })) {
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

async fn try_iccp_worker(
    id: String,
    name: String,
    state: Arc<RwLock<String>>,
    calling: AcseIdentityParameters,
    called: AcseIdentityParameters,
    event_listener: &mut EventQueue,
    notifier: &mut UnboundedReceiver<()>,
) -> Result<(), anyhow::Error> {
    *state.write().await = "Opening".into();
    // TODO registrations
    match event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssocaitionOperationalReport {
        id: id.clone(),
        name: name.clone(),
        acse_listener: None,
        status: "Opening".into(),
        calling: calling.clone(),
        called: called.clone(),
    })) {
        Ok(()) => (),
        Err(_) => return Ok(()),
    }

    *state.write().await = "Listening".into();
    // TODO Registrations
    match event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssocaitionOperationalReport { id: id.clone(), name: name.clone(), acse_listener: None, status: "Listening".into(), calling, called })) {
        Ok(()) => (),
        Err(_) => return Ok(()),
    }

    loop {
        tokio::select! {
            _ = notifier.recv() => return Ok(())
        };
    }
}
