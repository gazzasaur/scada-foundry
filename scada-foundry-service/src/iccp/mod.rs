use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    sync::Arc,
    time::Duration,
};

use num_bigint::BigInt;
use rand::random_range;
use rusty_mms::parameters::{
    ParameterSupportOption::{Str1, Str2, Vlis, Vnam},
    ServiceSupportOption::{Conclude, DefineNamedVariableList, DeleteNamedVariableList, GetNameList, GetNamedVariableListAttribute, GetVariableAccessAttributes, Identify, InformationReport, Read, Write},
};
use rusty_mms_service::{MmsServiceConnectionIdentityParameters, MmsServiceConnectionParameters, create_mms_service_client};
use tokio::sync::{
    RwLock,
    mpsc::{self, UnboundedReceiver, UnboundedSender, error::TryRecvError},
};

use crate::{
    api::ScadaFoundryEvent,
    error::ScadaFoundryError,
    iccp::{
        api::{
            IccpAssociation, IccpAssociationState, IccpAssociationStatus,
            IccpAssociationType::{ClientBidirectional, ClientUnidirectional},
        },
        converter::convert_object_identifiers,
    },
};

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

        let local_association = association.clone();
        let operator = Arc::new(RwLock::new(IccpSubsystemAssociationOperator { association: association.clone(), signalling_queue: signalling_receiver, terminate_queue: term_sender, listener: listener.clone() }));
        tokio::task::spawn(async move {
            loop {
                let send_result = match IccpSubsystemAssociationOperator::initialise(operator.clone()).await {
                    Ok(_) => break,
                    Err(e) => listener.send(ScadaFoundryEvent::IccpAssociationUpdate(IccpAssociationStatus {
                        id: local_association.clone().read().await.id.clone(),
                        name: local_association.read().await.name.clone(),
                        enabled: true,
                        status_description: format!("{e}"),
                        state: IccpAssociationState::Failed,
                    })),
                };
                if let Err(_) = send_result {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(10 + random_range(1..10))).await;
            }
        });
        return Self { association: association, signalling_queue: signalling_sender, terminate_queue: term_receiver };
    }

    pub async fn wait_for_terminate(mut self) {
        // We do not care about the results here. If the queue are already closed, so be it.
        let _ = self.signalling_queue.send(());
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
    async fn initialise(operator: Arc<RwLock<Self>>) -> Result<(), anyhow::Error> {
        match operator.write().await.signalling_queue.try_recv() {
            Ok(_) => return Ok(()),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => return Ok(()),
        }

        let association_lock = operator.read().await.association.clone();
        let association = association_lock.read().await;
        let host = match format!("{}:{}", association.host, association.port).parse() {
            Ok(host) => host,
            // TODO Indicate failure and retry.
            Err(_) => return Ok(()),
        };
        println!("{}", &association.local_data_center_parameters.ae_title.ae_qualifier);

        match operator.read().await.association.read().await.association_type {
            ClientUnidirectional | ClientBidirectional => {
                create_mms_service_client(
                    host,
                    MmsServiceConnectionParameters {
                        local_detail_calling: Some(4 * 1024 * 1024), // 4MB Payload Size. TODO Move to configuration

                        called: MmsServiceConnectionIdentityParameters {
                            tsap_id: Some(association.local_data_center_parameters.tsap.clone()),
                            session_selector: Some(association.local_data_center_parameters.ssap.clone()),
                            presentation_selector: Some(association.local_data_center_parameters.psap.clone()),

                            ap_title: Some(convert_object_identifiers(&association.local_data_center_parameters.ae_title.ap_title).unwrap()),
                            ae_qualifier: Some(vec![0]),
                            ap_invocation_identifier: None,
                            ae_invocation_identifier: None,
                        },
                        calling: MmsServiceConnectionIdentityParameters {
                            tsap_id: Some(association.remote_data_center_parameters.tsap.clone()),
                            session_selector: Some(association.remote_data_center_parameters.ssap.clone()),
                            presentation_selector: Some(association.remote_data_center_parameters.psap.clone()),

                            ap_title: Some(convert_object_identifiers(&association.remote_data_center_parameters.ae_title.ap_title).unwrap()),
                            ae_qualifier: Some(vec![0]),
                            ap_invocation_identifier: None,
                            ae_invocation_identifier: None,
                        },

                        proposed_max_serv_outstanding_calling: 1000,
                        proposed_max_serv_outstanding_called: 1000,
                        proposed_data_structure_nesting_level: Some(2),
                        propsed_parameter_cbb: vec![Str1, Str2, Vnam, Vlis],
                        services_supported_calling: vec![
                            GetNameList,
                            Identify,
                            Read,
                            Write,
                            GetVariableAccessAttributes,
                            DefineNamedVariableList,
                            GetNamedVariableListAttribute,
                            DeleteNamedVariableList,
                            InformationReport,
                            Conclude,
                        ],
                    },
                )
                .await
            }
            api::IccpAssociationType::ServerUnidirectional => todo!(),
            api::IccpAssociationType::ServerBidirectional => todo!(),
        }
        .unwrap();

        Ok(())

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
