use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    sync::{Arc, atomic::AtomicBool},
};

use rusty_iccp::{IccpClient, RustyIccpClient};
use rusty_mms::MmsInitiator;
use tokio::sync::{
    RwLock,
    mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender, error::TryRecvError},
};

use crate::{api::ScadaFoundryEvent, error::ScadaFoundryError, iccp::api::IccpAssociation};

pub mod api;
pub mod converter;

pub struct IccpSubsystemAssociationState {
    pub association: Arc<RwLock<IccpAssociation>>,
    pub signalling_queue: UnboundedSender<()>,
    pub terminate_queue: UnboundedReceiver<()>,
}

impl IccpSubsystemAssociationState {
    pub async fn new(listener: UnboundedSender<ScadaFoundryEvent>, association: Arc<RwLock<IccpAssociation>>) -> Self {
        let (signalling_sender, signalling_receiver) = mpsc::unbounded_channel();
        let (term_sender, term_receiver) = mpsc::unbounded_channel();

        let operator = Arc::new(RwLock::new(IccpSubsystemAssociationOperator { association: association.clone(), signalling_queue: signalling_receiver, terminate_queue: term_sender, listener }));
        tokio::task::spawn(IccpSubsystemAssociationOperator::initiate(operator));

        return Self { association: association, signalling_queue: signalling_sender, terminate_queue: term_receiver };
    }

    pub async fn wait_for_terminate(mut self) {
        self.signalling_queue.send(());
        self.terminate_queue.recv().await;
    }
}

pub struct IccpSubsystemAssociationOperator {
    pub association: Arc<RwLock<IccpAssociation>>,
    pub signalling_queue: UnboundedReceiver<()>,
    pub terminate_queue: UnboundedSender<()>,
    pub listener: UnboundedSender<ScadaFoundryEvent>,
}

impl IccpSubsystemAssociationOperator {
    pub async fn initiate(operator: Arc<RwLock<Self>>) {
        match operator.write().await.signalling_queue.try_recv() {
            Ok(_) => return,
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => return,
        }

        // RustyIccpClient::new(mms_client)
    }
}

pub struct IccpSubsystem {
    listener: UnboundedSender<ScadaFoundryEvent>,
    associations: HashMap<String, Arc<RwLock<IccpSubsystemAssociationState>>>,
    // fn set_listener(listener: Sender<IccpSubsystemEvent>);

    // async fn create_association(association: IccpAssociation);

    // async fn fetch_data_points() -> Vec<IccpDataPointValue>;

    // fn create_data_point(data_point: IccpDataPointValue);
    // fn update_data_point(data_point: IccpDataPointValue);
    // fn delete_data_point(data_point_key: IccpDataPointKey);
    // fn update_data_point_value(data_point_key: IccpDataPointKey, source: String, iccp_data_point_value: IccpDataPointValue);
}

impl IccpSubsystem {
    pub async fn new(listener: UnboundedSender<ScadaFoundryEvent>) -> Self {
        Self { listener, associations: HashMap::new() }
    }

    pub async fn list_associations(&self) -> Vec<String> {
        futures::future::join_all(self.associations.values().map(|assoc| async {
            // Clone here so we do not have to hold a lock while grabbing another lock. The risk of deadlock is too high.
            let association = assoc.read().await.association.clone();
            association.read().await.id.clone()
        }))
        .await
        .into_iter()
        .collect()
    }

    pub async fn create_association(&mut self, association: IccpAssociation) -> Result<(), ScadaFoundryError> {
        match self.associations.entry(association.id.clone()) {
            Occupied(_) => return Err(ScadaFoundryError::ApplicationError("association already exists".into())),
            Vacant(vacant_entry) => vacant_entry.insert(Arc::new(RwLock::new(IccpSubsystemAssociationState::new(self.listener.clone(), Arc::new(RwLock::new(association))).await))),
        };
        Ok(())
    }
}

async fn process_association_connect(mut association: IccpAssociation) {}
