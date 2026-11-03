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

use crate::{
    api::ScadaFoundryEvent,
    config::acse::AcseIdentityParameters,
    iccp::api::IccpAssociation,
};

pub type EventQueue = UnboundedSender<ScadaFoundryEvent>;
pub type ConnectionQueue = UnboundedSender<Box<dyn RustyMmsServiceServer>>;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IccpServerAssociationOperationalReport {
    pub association: IccpAssociation,
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

    pub async fn list_server_associations(&self) -> Vec<IccpServerAssociationOperationalReport> {
        let mut reports = vec![];
        for association in self.associations.read().await.values() {
            reports.push(IccpServerAssociationOperationalReport { status: association.status.read().await.clone(), association: association.association.clone() });
        }
        reports
    }

    pub async fn create_server_association(&mut self, assocation: IccpAssociation) -> IccpServerAssociationOperationalReport {
        let mut associations = self.associations.write().await;

        return match associations.entry(assocation.id.clone()) {
            Entry::Occupied(mut occupied_entry) => {
                let _ = occupied_entry.get()._notifier.send(());
                occupied_entry.get()._notifier.closed().await;

                let association_record = IccpServerAssociationOperationalReport { status: "Initialising".into(), association: occupied_entry.get().association.clone() };
                occupied_entry.insert(IccpAssociationManager::new(assocation, self.event_listener.clone()));
                association_record
            }
            Entry::Vacant(vacant_entry) => {
                let listener_record = IccpServerAssociationOperationalReport { status: "Initialising".into(), association: assocation.clone() };
                vacant_entry.insert_entry(IccpAssociationManager::new(assocation, self.event_listener.clone()));
                listener_record
            }
        };
    }

    pub async fn fetch_server_association(&self) -> Vec<IccpServerAssociationOperationalReport> {
    }

    pub async fn close(&mut self, id: String) {
        let mut associations = self.associations.write().await;

        return match associations.entry(id.clone()) {
            Entry::Occupied(occupied_entry) => {
                let _ = occupied_entry.get()._notifier.send(());
                occupied_entry.get()._notifier.closed().await;
                let item = occupied_entry.remove();
                let _ = self.event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssociationOperationalReport { status: "Deleted".into(), association: item.association.clone() }));
            }
            Entry::Vacant(_) => (),
        };
    }
}

pub struct IccpAssociationManager {
    // This will terminate the worked if the parent goes out of scope.
    _notifier: UnboundedSender<()>,
    association: IccpAssociation,
    status: Arc<RwLock<String>>,
}

impl IccpAssociationManager {
    pub fn new(association: IccpAssociation, event_listener: EventQueue) -> Self {
        let status = Arc::new(RwLock::new("Idle".into()));
        let (notifier_sender, notifier_receiver) = unbounded_channel();
        tokio::task::spawn(iccp_worker(association.clone(), status.clone(), event_listener, notifier_receiver));
        Self { status, association, _notifier: notifier_sender }
    }

    pub async fn state(&self) -> String {
        (*&self.status.read().await).clone()
    }
}

async fn iccp_worker(association: IccpAssociation, state: Arc<RwLock<String>>, acse_receiver: &mut UnboundedReceiver<Box<dyn RustyMmsServiceServer>>, mut event_listener: EventQueue, mut notifier: UnboundedReceiver<()>) {
    loop {
        *state.write().await = "Idle".into();
        // TODO Registrations
        match event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssociationOperationalReport { association: association.clone(), status: "Idle".into() })) {
            Ok(()) => (),
            Err(_) => return,
        }

        match try_iccp_worker(association.clone(), state.clone(), &mut event_listener, &mut notifier).await {
            Ok(()) => return,
            Err(e) => {
                error!("{e}");
                *state.write().await = "Error".into();
                // TODO Registrations
                match event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssociationOperationalReport { association: association.clone(), status: "Error".into() })) {
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

async fn try_iccp_worker(association: IccpAssociation, state: Arc<RwLock<String>>, event_listener: &mut EventQueue, notifier: &mut UnboundedReceiver<()>) -> Result<(), anyhow::Error> {
    *state.write().await = "Opening".into();
    // TODO registrations
    match event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssociationOperationalReport { association: association.clone(), status: "Opening".into() })) {
        Ok(()) => (),
        Err(_) => return Ok(()),
    }

    *state.write().await = "Listening".into();
    // TODO Registrations
    match event_listener.send(ScadaFoundryEvent::IccpServerAssociationReport(IccpServerAssociationOperationalReport { association: association.clone(), status: "Listening".into() })) {
        Ok(()) => (),
        Err(_) => return Ok(()),
    }

    loop {
        tokio::select! {
            _ = notifier.recv() => return Ok(())
        };
    }
}
