use std::{sync::Arc, time::Duration};

use rusty_iccp::{IccpOperation, IccpServer, RustyIccpServer};
use rusty_mms_service::RustyMmsServiceServer;
use tokio::{
    select,
    sync::{Notify, RwLock, mpsc::UnboundedReceiver},
};

use crate::{connectors::acse::EventQueue, iccp::api::IccpServerOperationalAssociation};

pub mod api;

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
    // event_listener.send(ScadaFoundryEvent::IccpAssociationUpdate(status.read().await.operational_associatiom.clone()));

    let mut iccp_server = RustyIccpServer::new(mms_server);
    loop {
        for operation in iccp_server.receive_operations().await? {
            match operation {
                IccpOperation::GetDataSetNames(operation) => todo!(),
                IccpOperation::CreateDataSet(operation) => todo!(),

                IccpOperation::Read(operation) => todo!(),

                IccpOperation::GetNextDsTransferSet(operation) => todo!(),
                IccpOperation::StartTransferSet(operation) => todo!(),

                IccpOperation::MmsAaRead(operation) => todo!(),
                IccpOperation::MmsOperation(operation) => todo!(),
            }
        }
    }
}
