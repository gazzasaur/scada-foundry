use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::future::join_all;
use rusty_iccp::{IccpOperation, IccpServer, RustyIccpServer};
use rusty_mms_service::RustyMmsServiceServer;
use tokio::{
    select,
    sync::{
        Notify, RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    },
};

use crate::{
    api::ScadaFoundryEvent, connectors::acse::EventQueue, error::ScadaFoundryError, iccp::api::{IccpAssociation, IccpAssociationStatus, IccpServerOperationalAssociation},
};

pub mod api;
pub mod converter;
pub mod tpkt;

pub struct IccpSubsystem {
    event_listener: EventQueue,
    associations: Arc<RwLock<HashMap<String, Arc<RwLock<IccpServerAssociationRegistration>>>>>,
}

impl IccpSubsystem {
    pub fn new(event_listener: EventQueue) -> Self {
        Self { event_listener, associations: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn list_server_associations(&self) -> Vec<IccpServerOperationalAssociation> {
        let associations: Vec<_> = self.associations.read().await.values().map(|x| x.clone()).collect();
        let association_statuses: Vec<_> = join_all(associations.iter().map(|x| x.read())).await.iter().map(|x| (*x).operational_associatiom.clone()).collect();
        association_statuses
    }

    pub async fn register_server_association(&mut self, association: IccpAssociation) -> UnboundedSender<Box<dyn RustyMmsServiceServer>> {
        let (acse_sender, acse_receiver) = unbounded_channel();
        let id = association.id.clone();
        let status = Arc::new(RwLock::new(IccpServerAssociationRegistration { notifier: Arc::new(Notify::new()), operational_associatiom: IccpServerOperationalAssociation { state: "Acting".into(), status: "Idle".into(), association } }));
        self.associations.write().await.insert(id, status.clone());
        tokio::task::spawn(iccp_worker(status, acse_receiver, self.event_listener.clone()));
        acse_sender
    }
}

struct IccpServerAssociationRegistration {
    notifier: Arc<Notify>,
    operational_associatiom: IccpServerOperationalAssociation,
}

async fn iccp_worker(status: Arc<RwLock<IccpServerAssociationRegistration>>, mut acse_receiver: UnboundedReceiver<Box<dyn RustyMmsServiceServer>>, mut event_listener: EventQueue) {
    loop {
        match try_iccp_worker(&status, &mut acse_receiver, &mut event_listener).await {
            Ok(()) => return,
            Err(_) => tokio::time::sleep(Duration::from_secs(2)).await,
        }
    }
}

async fn try_iccp_worker(status: &Arc<RwLock<IccpServerAssociationRegistration>>, acse_receiver: &mut UnboundedReceiver<Box<dyn RustyMmsServiceServer>>, event_listener: &mut EventQueue) -> Result<(), anyhow::Error> {
    let notifier = status.read().await.notifier.clone();

    let mms_server = select! {
        _ = notifier.notified() => return Ok(()),
        x = acse_receiver.recv() => match x {
            Some(x) => x,
            None => return Ok(()),
        }
    };
    status.write().await.operational_associatiom.status = "Connecting".into();
    event_listener.send(ScadaFoundryEvent::IccpAssociationUpdate(status.read().await.operational_associatiom.clone()));

    let iccp_server = RustyIccpServer::new(mms_server, domain, bilateral_table_id);
    loop {
        match iccp_server.receive_operation().await {
            Ok(op) => match op {
                IccpOperation::MmsOperation(mms_service_message) => todo!(),
                IccpOperation::StartTransferSet(start_transfer_set_operation) => todo!(),
                IccpOperation::CreateDataSet(create_data_set_operation) => todo!(),
                IccpOperation::GetDataSetNames(get_data_set_names_operation) => todo!(),
                IccpOperation::GetNextDsTransferSet(get_next_ds_transfer_set_operation) => todo!(),
            },
            Err(e) => Err(e)?,
        }
    }

    Ok(())
}
