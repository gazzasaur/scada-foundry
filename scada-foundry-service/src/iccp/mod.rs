use std::{
    error::Error, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration
};

use rusty_iccp::RustyIccpClient;
use rusty_mms_service::{RustyMmsServiceFactory, RustyTpktClientConnectionFactory, TpktClientConnectionFactory, datapump::MmsServiceDataPump};
use tokio::sync::{Mutex, mpsc::{self, UnboundedReceiver, error::TryRecvError}};
use tokio::sync::{RwLock, mpsc::UnboundedSender};

use crate::config::iccp::{IccpConfiguration, InitiatorIccpAssociation};

enum IccpConfigure {
    CreateInitiator {
        listeners: Arc<RwLock<Vec<UnboundedSender<IccpEvent>>>>,
        association: InitiatorIccpAssociation
    },
    Close,
}

pub enum IccpEvent {
    State { uuid: String, association: String, state: String },
}

pub struct IccpManager {
    configure: UnboundedSender<IccpConfigure>,
    listeners: Arc<RwLock<Vec<UnboundedSender<IccpEvent>>>>,
}

impl IccpManager {
    pub fn new(config: IccpConfiguration) -> Self {
        let listeners = Arc::new(RwLock::new(Vec::new()));
        let (configure_sender, configure_receiver) = mpsc::unbounded_channel();

        let bindings = Arc::new(Mutex::new(Vec::new()));
        let data_pump = Arc::new(MmsServiceDataPump::new(Arc::new(AtomicBool::new(true), ), bindings));

        let mut worker = IccpManagerWorker { configure: configure_receiver, listeners: listeners.clone(), data_pump };
        tokio::task::spawn(async move {
            worker.process();
        });

        for initiator in config.initator_associations {
            configure_sender.send(IccpConfigure::CreateInitiator(initiator.clone()));
        }

        IccpManager { configure: configure_sender, listeners }
    }
}

impl Drop for IccpManager {
    fn drop(&mut self) {
        self.configure.send(IccpConfigure::Close);
    }
}

enum InitiatorAssociationState {
    New(InitiatorIccpAssociation),
    // Connecting(InitiatorIccpAssociation, UnboundedReceiver<Result<RustyIccpClient, Box<dyn Error>>>),
    // Connected(InitiatorIccpAssociation, RustyIccpClient),
}

struct IccpManagerWorker {
    data_pump: Arc<MmsServiceDataPump>,

    configure: UnboundedReceiver<IccpConfigure>,
    listeners: Arc<RwLock<Vec<UnboundedSender<IccpEvent>>>>,
}

impl IccpManagerWorker {
    async fn process(&mut self) {
        let mut initiator_associations: Vec<InitiatorAssociationState> = Vec::new();

        loop {
            match self.configure.try_recv() {
                Ok(IccpConfigure::Close) => return,
                Ok(IccpConfigure::CreateInitiator(initiator)) => {
                    initiator_associations.push(InitiatorAssociationState::New(initiator));
                }

                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return,
            }

            for initiator_association in initiator_associations.drain(..) {
                match initiator_association {
                    InitiatorAssociationState::New(initiator_iccp_association) => {
                        tokio::task::spawn(iccp_initiator_connect(self.data_pump.clone(), initiator_iccp_association));
                    },
                    // InitiatorAssociationState::Connecting(initiator_iccp_association) => todo!(),
                    // InitiatorAssociationState::Connected(initiator_iccp_association, rusty_iccp_client) => todo!(),
                }
            }
        }
    }
}

// TODO Allow connections to use separate or the same data pumps
async fn iccp_initiator_connect(data_pump: Arc<MmsServiceDataPump>, initiator_iccp_association: InitiatorIccpAssociation) {
    if let Err(e) = try_iccp_initiator_connect(data_pump, initiator_iccp_association).await {

    }
}

async fn try_iccp_initiator_connect(data_pump: Arc<MmsServiceDataPump>, initiator_iccp_association: InitiatorIccpAssociation) -> Result<(), anyhow::Error> {
    let host = initiator_iccp_association.remote_control_center.host;
    let port = initiator_iccp_association.remote_control_center.port;
    let socket_address = format!("{host}:{port}");
    let factory = RustyMmsServiceFactory::new(data_pump);
    RustyTpktClientConnectionFactory::new(socket_address.into()?);
    factory.create_client_connection(tpkt_connection_factory, parameters);
    Ok(())
}