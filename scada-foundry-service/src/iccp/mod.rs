use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    sync::Arc,
    time::Duration,
};

use rand::random_range;
use rusty_mms::parameters::{
    ParameterSupportOption::{Str1, Str2, Vlis, Vnam},
    ServiceSupportOption::{Conclude, DefineNamedVariableList, DeleteNamedVariableList, GetNameList, GetNamedVariableListAttribute, GetVariableAccessAttributes, Identify, InformationReport, Read, Write},
};
use rusty_mms_service::{
    MmsServiceConnectionIdentityParameters, MmsServiceConnectionParameters, create_mms_service_client, create_mms_service_server,
    error::{self, MmsServiceError},
};
use tokio::sync::{
    RwLock,
    mpsc::{self, UnboundedReceiver, UnboundedSender, error::TryRecvError},
};
use tracing::error;

use crate::{
    api::ScadaFoundryEvent,
    error::ScadaFoundryError,
    iccp::{
        OperatorError::{Fatal, Retry},
        api::{
            IccpAssociation, IccpAssociationState, IccpAssociationStatus,
            IccpAssociationType::{ClientBidirectional, ClientUnidirectional, ServerBidirectional, ServerUnidirectional},
        },
        converter::convert_object_identifiers,
    },
};

pub mod api;
pub mod converter;

pub struct IccpServerConnections {}

pub struct IccpSubsystemAssociationState {
    pub association: Arc<RwLock<IccpAssociationState>>,
    pub signalling_queue: UnboundedSender<()>,
    pub terminate_queue: UnboundedReceiver<()>,
}

impl IccpSubsystemAssociationState {
    pub async fn new(listener: UnboundedSender<ScadaFoundryEvent>, association: Arc<RwLock<IccpAssociationState>>, server_connections: Arc<RwLock<IccpServerConnections>>) -> Self {
        let (signalling_sender, signalling_receiver) = mpsc::unbounded_channel();
        let (term_sender, term_receiver) = mpsc::unbounded_channel();

        let local_association = association.clone();
        let operator = Arc::new(RwLock::new(IccpSubsystemAssociationOperator {
            association_state: association.clone(),
            signalling_queue: signalling_receiver,
            terminate_queue: term_sender,
            listener: listener.clone(),
            server_connections: server_connections.clone(),
        }));
        tokio::task::spawn(async move {
            loop {
                let send_result = match IccpSubsystemAssociationOperator::initialise(operator.clone()).await {
                    Ok(()) => break, // Finish normally
                    Err(Fatal(reason)) => {
                        error!("ICCP Connection has terminated abnormally: {reason}");
                        local_association.write().await.status = IccpAssociationStatus::Failed;
                        listener.send(ScadaFoundryEvent::IccpAssociationUpdate(local_association.read().await.clone()));
                        break;
                    }
                    Err(Retry(cause)) => {
                        error!("ICCP Connection has failed and will retry: {cause}");
                        local_association.write().await.status = IccpAssociationStatus::Failed;
                        listener.send(ScadaFoundryEvent::IccpAssociationUpdate(local_association.read().await.clone()))
                    }
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

pub enum OperatorError {
    Fatal(String),
    Retry(anyhow::Error),
}

// TODO This association should probably be a clone so we can reduce locking inside a critical task.
pub struct IccpSubsystemAssociationOperator {
    pub terminate_queue: UnboundedSender<()>,
    pub signalling_queue: UnboundedReceiver<()>,
    pub listener: UnboundedSender<ScadaFoundryEvent>,
    pub association_state: Arc<RwLock<IccpAssociationState>>,
    pub server_connections: Arc<RwLock<IccpServerConnections>>,
}

impl IccpSubsystemAssociationOperator {
    async fn initialise(operator: Arc<RwLock<Self>>) -> Result<(), OperatorError> {
        // Grab some local data that should not change during the connection cycle.
        let (local_listener, local_association) = {
            let operator_lock = operator.read().await;
            let local_listener = operator_lock.listener.clone();
            let local_association = operator_lock.association_state.read().await.association.clone();
            (local_listener, local_association)
        };

        // Check to see if we are closed.
        match operator.write().await.signalling_queue.try_recv() {
            Ok(_) => return Ok(()),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => return Ok(()),
        }

        Self::update_state(operator.clone(), local_listener.clone(), IccpAssociationStatus::Idle).await?;

        match local_association.association_type {
            ClientUnidirectional => Self::client_unidirectional(operator, local_listener, local_association).await?,
            ServerUnidirectional => Self::server_unidirectional(operator, local_listener, local_association).await?,
            ClientBidirectional => return Err(Fatal("ClientBidirectional Not Implemented".into())),
            ServerBidirectional => return Err(Fatal("ServerBidirectional Not Implemented".into())),
        };
        Ok(())
    }

    async fn client_unidirectional(operator: Arc<RwLock<Self>>, listener: UnboundedSender<ScadaFoundryEvent>, association: IccpAssociation) -> Result<(), OperatorError> {
        let host = match format!("{}:{}", association.host, association.port).parse() {
            Ok(host) => host,
            Err(e) => {
                error!("Invalid address: {}:{}", association.host, association.port);
                Self::update_state(operator.clone(), listener.clone(), IccpAssociationStatus::Failed).await?;
                return Err(Fatal(format!("{e}")));
            }
        };
        Self::update_state(operator.clone(), listener.clone(), IccpAssociationStatus::Connecting).await?;

        let mut client = create_mms_service_client(
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
        .map_err(|e| Retry(anyhow::anyhow!("{e}")))?;
        Self::update_state(operator.clone(), listener.clone(), IccpAssociationStatus::Connected).await?;

        let _ = client.receive_information_report().await;

        Ok(())
    }

    async fn server_unidirectional(operator: Arc<RwLock<Self>>, listener: UnboundedSender<ScadaFoundryEvent>, association: IccpAssociation) -> Result<(), OperatorError> {
        let host = match format!("{}:{}", association.host, association.port).parse() {
            Ok(host) => host,
            Err(e) => {
                error!("Invalid address: {}:{}", association.host, association.port);
                Self::update_state(operator.clone(), listener.clone(), IccpAssociationStatus::Failed).await?;
                return Err(Fatal(format!("{e}")));
            }
        };

        Self::update_state(operator.clone(), listener.clone(), IccpAssociationStatus::Connecting).await?;
        let mut server = create_mms_service_server(
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
        .map_err(|e| Retry(anyhow::anyhow!("{e}")))?;
        Self::update_state(operator.clone(), listener.clone(), IccpAssociationStatus::Connected).await?;

        let _ = server.receive_message().await;

        Ok(())
    }

    async fn update_state(operator: Arc<RwLock<IccpSubsystemAssociationOperator>>, listener: UnboundedSender<ScadaFoundryEvent>, status: IccpAssociationStatus) -> Result<(), OperatorError> {
        let operator_lock = operator.read().await;
        let mut association_state = operator_lock.association_state.write().await;
        association_state.status = status;
        if let Err(x) = listener.send(ScadaFoundryEvent::IccpAssociationUpdate(association_state.clone())) {
            error!("Failed to send event to global listener: {x}");
            return Err(OperatorError::Fatal(format!("{x}")));
        }
        return Ok(());
    }
}

pub struct IccpSubsystem {
    listener: UnboundedSender<ScadaFoundryEvent>,
    server_connections: Arc<RwLock<IccpServerConnections>>,
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
        Self { listener, server_connections: Arc::new(RwLock::new(IccpServerConnections {})), associations: HashMap::new() }
    }

    pub async fn list_associations(&self) -> Vec<IccpAssociationState> {
        futures::future::join_all(self.associations.values().map(|assoc| async {
            // Clone here so we do not have to hold a lock while grabbing another lock. The risk of deadlock is too high.
            let association = assoc.read().await.association.clone();
            association.read().await.clone()
        }))
        .await
        .into_iter()
        .collect()
    }

    pub async fn create_association(&mut self, association: IccpAssociation) -> () {
        let state = IccpAssociationState { association: association, status: IccpAssociationStatus::Idle };
        match self.associations.entry(state.association.id.clone()) {
            Occupied(mut entry) => {
                entry.insert(Arc::new(RwLock::new(IccpSubsystemAssociationState::new(self.listener.clone(), Arc::new(RwLock::new(state)), self.server_connections.clone()).await)));
            }
            Vacant(vacant_entry) => {
                vacant_entry.insert(Arc::new(RwLock::new(IccpSubsystemAssociationState::new(self.listener.clone(), Arc::new(RwLock::new(state)), self.server_connections.clone()).await)));
            }
        };
    }
}
