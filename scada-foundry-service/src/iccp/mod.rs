use std::{
    error::Error, sync::{Arc, atomic::Ordering}, time::Duration
};

use rusty_iccp::RustyIccpClient;
use tokio::sync::mpsc::{self, UnboundedReceiver, error::TryRecvError};
use tokio::sync::{RwLock, mpsc::UnboundedSender};

use crate::config::iccp::{IccpConfiguration, InitiatorIccpAssociation};

enum IccpConfigure {
    CreateInitiator(InitiatorIccpAssociation),
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

        let mut worker = IccpManagerWorker { configure: configure_receiver, listeners: listeners.clone() };
        tokio::task::spawn(async move {
            worker.process();
        });

        for initiator in config.initator_associations {
            configure_sender.send(IccpConfigure::CreateInitiator(initiator.clone()));
        }

        IccpManager { configure: configure_sender, listeners }
    }
}

enum InitiatorAssociationState {
    New(InitiatorIccpAssociation),
    Connecting(InitiatorIccpAssociation, UnboundedReceiver<Result<RustyIccpClient, Box<dyn Error>>>),
    // Connected(InitiatorIccpAssociation, RustyIccpClient),
}

struct IccpManagerWorker {
    configure: UnboundedReceiver<IccpConfigure>,
    listeners: Arc<RwLock<Vec<UnboundedSender<IccpEvent>>>>,
}

impl IccpManagerWorker {
    async fn process(&mut self) {
        let mut initiator_associations: Vec<InitiatorAssociationState> = Vec::new();

        loop {
            match self.configure.try_recv() {
                Ok(IccpConfigure::CreateInitiator(initiator)) => {
                    initiator_associations.push(InitiatorAssociationState::New(initiator));
                }

                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return,
            }

            for initiator_association in &initiator_associations {
                match initiator_association {
                    InitiatorAssociationState::New(initiator_iccp_association) => {
                        tokio::task::spawn(future)
                    },
                    InitiatorAssociationState::Connecting(initiator_iccp_association) => todo!(),
                    // InitiatorAssociationState::Connected(initiator_iccp_association, rusty_iccp_client) => todo!(),
                }
            }
        }
    }
}

async fn iccp_initiator_connect() -> Result<RustyIccpClient, Box<dyn Error>> {
    
}