use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
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

impl Drop for IccpManager {
    fn drop(&mut self) {
        self.running.store(false, Ordering::AcqRel);
    }
}

struct IccpManagerWorker {
    configure: UnboundedReceiver<IccpConfigure>,
    listeners: Arc<RwLock<Vec<UnboundedSender<IccpEvent>>>>,
}

impl IccpManagerWorker {
    async fn process(&mut self) {
        let initiator_associations: Vec<RustyIccpClient> = Vec::new();

        loop {
            match self.configure.try_recv() {
                Ok(IccpConfigure::CreateInitiator(initiator)) => {

                },

                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return,
            }
        }
    }
}
