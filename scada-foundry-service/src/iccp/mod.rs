pub mod api;

use bigdecimal::BigDecimal;
use der_parser::Oid;
use der_parser::asn1_rs::Any;
use der_parser::der::Tag;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use num_bigint::BigInt;
use oid::ObjectIdentifier;
use rusty_mms_service::datapump::MmsServiceDataPump;
use rusty_mms_service::{MmsInitiatorService, MmsResponderService, MmsServiceConnectionIdentityParameters, MmsServiceConnectionParameters, RustyMmsServiceClient, RustyMmsServiceFactory, RustyMmsServiceServer, RustyTpktClientConnectionFactory, RustyTpktServerConnectionFactory};
use rusty_tpkt::{TcpTpktConnection, TcpTpktReader, TcpTpktWriter};
use std::collections::HashMap;
use std::collections::hash_map::Entry::Vacant;
use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::error;
use uuid::Uuid;
use rusty_iccp::IccpData;

use tokio::sync::{Mutex, RwLock};

use crate::config::iccp::{InitiatorIccpAssociation, ResponderIccpAssociation};

pub struct IccpManagerAssociation {
    pub valid: Arc<AtomicBool>,
}

impl IccpManagerAssociation {
    pub fn new(valid: Arc<AtomicBool>) -> Self {
        Self { valid }
    }
}

impl Drop for IccpManagerAssociation {
    fn drop(&mut self) {
        self.valid.store(false, Ordering::SeqCst);
    }
}

enum IccpManagerWorkerEvent {
    InitiatorIccpAssociation { state: Arc<RwLock<IccpManagerState>>, uuid: String },
    ResponderIccpAssociation { state: Arc<RwLock<IccpManagerState>>, uuid: String },

    IccpServerDataTask { state: Arc<RwLock<IccpManagerState>>, uuid: String, client: Box<dyn RustyMmsServiceServer + 'static> },
}

// FIXME: Use state enum
#[derive(Clone)]
pub enum IccpManagerEvent {
    IccpAssociationStateUpdate(String, String), // State, Reason
}

struct IccpManagerState {
    initiators: HashMap<String, Arc<RwLock<InitiatorIccpAssociationState>>>,
    responders: HashMap<String, Arc<RwLock<ResponderIccpAssociationState>>>,
}

#[derive(Clone)]
struct InitiatorIccpAssociationState {
    uuid: String,
    association: InitiatorIccpAssociation,
    listener: UnboundedSender<IccpManagerEvent>,
}

#[derive(Clone)]
struct ResponderIccpAssociationState {
    uuid: String,
    association: ResponderIccpAssociation,
    listener: UnboundedSender<IccpManagerEvent>,
}

impl IccpManagerState {
    fn new() -> Self {
        IccpManagerState { initiators: HashMap::new(), responders: HashMap::new() }
    }
}

#[derive(Clone)]
pub struct IccpManager {
    iccp_manager_state: Arc<RwLock<IccpManagerState>>,

    task_queue: UnboundedSender<IccpManagerWorkerEvent>,
    receive_queue: Arc<Mutex<UnboundedReceiver<IccpManagerWorkerEvent>>>,
}

impl IccpManager {
    pub async fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        IccpManager { iccp_manager_state: Arc::new(RwLock::new(IccpManagerState::new())), task_queue: sender, receive_queue: Arc::new(Mutex::new(receiver)) }
    }

    pub async fn initiator_iccp_association(&self, association_information: InitiatorIccpAssociation) -> Result<UnboundedReceiver<IccpManagerEvent>, anyhow::Error> {
        loop {
            let uuid = Uuid::new_v4().to_string();
            let state = self.iccp_manager_state.clone();
            let mut iccp_manager_state = self.iccp_manager_state.write().await;
            let (sender, receiver) = unbounded_channel();

            if let Vacant(vacant_entry) = iccp_manager_state.initiators.entry(uuid.clone()) {
                vacant_entry.insert_entry(Arc::new(RwLock::new(InitiatorIccpAssociationState { uuid: uuid.clone(), association: association_information, listener: sender })));
                self.task_queue.send(IccpManagerWorkerEvent::InitiatorIccpAssociation { state, uuid }).map_err(|e| anyhow::anyhow!("{e}"))?;
                return Ok(receiver);
            }
        }
    }

    pub async fn responder_iccp_association(&self, association_information: ResponderIccpAssociation) -> Result<UnboundedReceiver<IccpManagerEvent>, anyhow::Error> {
        loop {
            let uuid = Uuid::new_v4().to_string();
            let state = self.iccp_manager_state.clone();
            let mut iccp_manager_state = self.iccp_manager_state.write().await;
            let (sender, receiver) = unbounded_channel();

            if let Vacant(vacant_entry) = iccp_manager_state.responders.entry(uuid.clone()) {
                vacant_entry.insert_entry(Arc::new(RwLock::new(ResponderIccpAssociationState { uuid: uuid.clone(), association: association_information, listener: sender })));
                self.task_queue.send(IccpManagerWorkerEvent::ResponderIccpAssociation { state, uuid }).map_err(|e| anyhow::anyhow!("{e}"))?;
                return Ok(receiver);
            }
        }
    }

    // TODO Each thread will have it's own task runner, so it isn't really load balanced.
    pub async fn serve(&self) -> Result<(), anyhow::Error> {
        let mut task_runner = FuturesUnordered::new();

        loop {
            let mut receive_queue = self.receive_queue.lock().await;
            tokio::select! {
                task = receive_queue.recv() => {
                    drop(receive_queue);
                    match task {
                        Some(x) => task_runner.push(iccp_task(x, self.task_queue.clone())),
                        None => return Ok(()),
                    }
                }
                _ = task_runner.next() => {}
            }

            // let task = self.receive_queue.lock().await.try_recv();

            // tokio::ti task_runner.next().await;
        }
    }
}

async fn iccp_task(task_info: IccpManagerWorkerEvent, receive_queue: UnboundedSender<IccpManagerWorkerEvent>) {
    match task_info {
        IccpManagerWorkerEvent::InitiatorIccpAssociation { state, uuid } => {
            iccp_initiator_task(state, uuid).await;
        }
        IccpManagerWorkerEvent::ResponderIccpAssociation { state, uuid } => {
            match iccp_responder_task(state.clone(), uuid.clone()).await {
                Some(x) => {
                    receive_queue.send(IccpManagerWorkerEvent::IccpServerDataTask { state: state.clone(), uuid: uuid.clone(), client: x }).unwrap();
                },
                None => return,
            }
        }
        IccpManagerWorkerEvent::IccpServerDataTask { state, uuid, mut client } => {
            client.receive_message().await.unwrap();
        },
    }
}

async fn iccp_initiator_task(state: Arc<RwLock<IccpManagerState>>, uuid: String) {
    loop {
        match try_iccp_initiator_task(state.clone(), uuid.clone()).await {
            Ok(_) => return,
            Err(e) => {
                error!("Failed to connect: {e}");
                tokio::time::sleep(Duration::from_millis(3000)).await;
            }
        };
    }
}

async fn try_iccp_initiator_task(state: Arc<RwLock<IccpManagerState>>, uuid: String) -> Result<Option<Box<dyn RustyMmsServiceClient>>, anyhow::Error> {
    let initiator_iccp_association_state_container = match state.read().await.initiators.get(&uuid) {
        Some(value) => value.clone(),
        None => return Ok(None),
    };
    let initiator_iccp_association = initiator_iccp_association_state_container.read().await.association.clone();

    let running = Arc::new(AtomicBool::new(true));
    let bindings: Arc<Mutex<Vec<Pin<Box<dyn Future<Output = ()> + Send>>>>> = Arc::new(Mutex::new(vec![]));

    let remote_host = initiator_iccp_association.remote_control_center.host.clone();
    let remote_port = initiator_iccp_association.remote_control_center.port.clone();
    let socket_address = format!("{remote_host}:{remote_port}");

    let local_control_center = initiator_iccp_association.local_control_center.clone();
    let local_ap_title = convert_object_identifier_reported("Failed", "Invalid Local AP Title", initiator_iccp_association_state_container.clone(), &local_control_center.ae_title.ap_title).await?;
    let local_ae_qualifier = convert_bigdecimal_to_bigint_reported("Failed", "Invalid Local AE Qualifier", initiator_iccp_association_state_container.clone(), &local_control_center.ae_title.ae_qualifier).await?;

    let remote_control_center = initiator_iccp_association.remote_control_center.clone();
    let remote_ap_title = convert_object_identifier_reported("Failed", "Invalid Local AP Title", initiator_iccp_association_state_container.clone(), &remote_control_center.ae_title.ap_title).await?;
    let remote_ae_qualifier = convert_bigdecimal_to_bigint_reported("Failed", "Invalid Local AE Qualifier", initiator_iccp_association_state_container.clone(), &remote_control_center.ae_title.ae_qualifier).await?;

    let mut mms_parameters = MmsServiceConnectionParameters::default();
    mms_parameters.calling = MmsServiceConnectionIdentityParameters {
        tsap_id: Some(initiator_iccp_association.local_control_center.tsap_address),
        session_selector: Some(initiator_iccp_association.local_control_center.ssap_address),
        presentation_selector: Some(initiator_iccp_association.local_control_center.psap_address),
        ap_title: Some(local_ap_title),
        ae_qualifier: Some(local_ae_qualifier.to_signed_bytes_be()),
        ap_invocation_identifier: None,
        ae_invocation_identifier: None,
    };
    mms_parameters.called = MmsServiceConnectionIdentityParameters {
        tsap_id: Some(initiator_iccp_association.remote_control_center.tsap_address),
        session_selector: Some(initiator_iccp_association.remote_control_center.ssap_address),
        presentation_selector: Some(initiator_iccp_association.remote_control_center.psap_address),
        ap_title: Some(remote_ap_title),
        ae_qualifier: Some(remote_ae_qualifier.to_signed_bytes_be()),
        ap_invocation_identifier: None,
        ae_invocation_identifier: None,
    };

    Ok(Some(rusty_mms_service::create_mms_service_client(socket_address.parse()?, mms_parameters).await?))
}

async fn iccp_responder_task(state: Arc<RwLock<IccpManagerState>>, uuid: String) -> Option<Box<dyn RustyMmsServiceServer>> {
    loop {
        match try_iccp_responder_task(state.clone(), uuid.clone()).await {
            Ok(x) => return x,
            Err(e) => {
                error!("Unhandled Error: {e}");
                tokio::time::sleep(Duration::from_millis(3000)).await;
            }
        };
    }
}

// FIXME: This should allow multiple associations to use the same TCP connection
async fn try_iccp_responder_task(state: Arc<RwLock<IccpManagerState>>, uuid: String) -> Result<Option<Box<dyn RustyMmsServiceServer>>, anyhow::Error> {
    let responder_iccp_association = match state.read().await.responders.get(&uuid) {
        Some(value) => value.clone(),
        None => return Ok(None),
    };

    // let remote_host = responder_iccp_association.local_matcher.po;
    // let remote_port = responder_iccp_association.remote_control_center.port;
    let socket_address = "0.0.0.0:8102".parse()?;

    Ok(Some(rusty_mms_service::create_mms_service_server(socket_address, MmsServiceConnectionParameters::default()).await?))
}

// struct ScheduledIccpManagerWorkerTask {
//     delay: u64,
//     task: IccpManagerWorkerTask,
// }

// impl Eq for ScheduledIccpManagerWorkerTask {}

// impl PartialEq for ScheduledIccpManagerWorkerTask {
//     fn eq(&self, other: &Self) -> bool {
//         self.scheduled_instant == other.scheduled_instant
//     }
// }

// impl Ord for ScheduledIccpManagerWorkerTask {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.scheduled_instant.cmp(&other.scheduled_instant)
//     }
// }

// impl PartialOrd for ScheduledIccpManagerWorkerTask {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.scheduled_instant.partial_cmp(&other.scheduled_instant)
//     }
// }

// enum InitiatorAssociationState {
//     New(InitiatorIccpAssociation),
//     // Connecting(InitiatorIccpAssociation, UnboundedReceiver<Result<RustyIccpClient, Box<dyn Error>>>),
//     // Connected(InitiatorIccpAssociation, RustyIccpClient),
// }

// struct IccpManagerWorker {
//     data_pump: Arc<MmsServiceDataPump>,
//     task_queue: UnboundedReceiver<IccpManagerWorkerTask>,

//     scheduled_tasks: BinaryHeap<Reverse<ScheduledIccpManagerWorkerTask>>,
// }

// impl IccpManagerWorker {
//     async fn new(task_queue: UnboundedReceiver<IccpManagerWorkerTask>) -> Self {
//         let data_pump = Arc::new(MmsServiceDataPump::new(Arc::new(AtomicBool::new(true)), Arc::new(Mutex::new(Vec::new()))));
//         IccpManagerWorker { data_pump, task_queue, scheduled_tasks: BinaryHeap::new() }
//     }

//     async fn process(&mut self) {
//         // let initiator_associations: Arc<Mutex<Vec<InitiatorAssociationState>>> = Arc::new(Mutex::new(Vec::new()));
//         // let mut current_bindings = FuturesUnordered::new();

//         let mut next_event = Instant::now();
//         loop {
//             let wait_period = Duration::min(Duration::max(next_event - Instant::now(), Duration::from_millis(0)), Duration::from_millis(1));

//             match self.task_queue.recv().await {
//                 Some(task) => self.scheduled_tasks.push(Reverse(ScheduledIccpManagerWorkerTask { task, scheduled_instant: Instant::now().add(Duration::from_secs(rand::random())) })),
//                 None => break,
//             }

//             // match self.configure.try_recv() {
//             //     Ok(IccpWorkerRequest::Close) => return,
//             //     Ok(IccpWorkerRequest::CreateInitiator { listeners, association }) => {
//             //         initiator_associations.lock().await.push(InitiatorAssociationState::New(association));
//             //     }

//             //     Err(TryRecvError::Empty) => (),
//             //     Err(TryRecvError::Disconnected) => return,
//             // }

//             // let fgh = {
//             //     let mut abc = initiator_associations.lock().await;
//             //     let abcd: Vec<InitiatorAssociationState> = abc.drain(..).collect();
//             //     abcd
//             // };

//             // for initiator_association in fgh {
//             //     match initiator_association {
//             //         InitiatorAssociationState::New(initiator_iccp_association) => {
//             //             let t1 = iccp_initiator_connect(self.data_pump.clone(), initiator_iccp_association.clone());
//             //             current_bindings.push(t1);
//             //             current_bindings.next().await;
//             //             // iccp_initiator_connect(dp, config).await;
//             //         } // InitiatorAssociationState::Connecting(initiator_iccp_association) => todo!(),
//             //           // InitiatorAssociationState::Connected(initiator_iccp_association, rusty_iccp_client) => todo!(),
//             //     }
//             // }
//         }
//     }
// }

// // TODO Allow arbitrary configuration of the datapump and connection mapping
// async fn iccp_initiator_connect(data_pump: Arc<MmsServiceDataPump>, initiator_iccp_association: InitiatorIccpAssociation) {
//     if let Err(e) = try_iccp_initiator_connect(data_pump, initiator_iccp_association).await {
//         error!("Failed to create connection: {}", e);
//     }
// }

// async fn try_iccp_initiator_connect(data_pump: Arc<MmsServiceDataPump>, initiator_iccp_association: InitiatorIccpAssociation) -> Result<(), anyhow::Error> {
//     let host = initiator_iccp_association.remote_control_center.host;
//     let port = initiator_iccp_association.remote_control_center.port;

//     let mut mms_parameters = MmsServiceConnectionParameters::default();
//     mms_parameters.calling = MmsServiceConnectionIdentityParameters {
//         tsap_id: Some(initiator_iccp_association.local_control_center.tsap_address),
//         session_selector: Some(initiator_iccp_association.local_control_center.ssap_address),
//         presentation_selector: Some(initiator_iccp_association.local_control_center.psap_address),
//         ap_title: Some(convert_object_identifiers(&initiator_iccp_association.local_control_center.ae_title.ap_title)?),
//         ae_qualifier: Some(convert_bigdecimal_to_bigint(&initiator_iccp_association.local_control_center.ae_title.ae_qualifier)?.to_signed_bytes_be()),
//         ap_invocation_identifier: None,
//         ae_invocation_identifier: None,
//     };
//     mms_parameters.called = MmsServiceConnectionIdentityParameters {
//         tsap_id: Some(initiator_iccp_association.remote_control_center.tsap_address),
//         session_selector: Some(initiator_iccp_association.remote_control_center.ssap_address),
//         presentation_selector: Some(initiator_iccp_association.remote_control_center.psap_address),
//         ap_title: Some(convert_object_identifiers(&initiator_iccp_association.remote_control_center.ae_title.ap_title)?),
//         ae_qualifier: Some(convert_bigdecimal_to_bigint(&initiator_iccp_association.remote_control_center.ae_title.ae_qualifier)?.to_signed_bytes_be()),
//         ap_invocation_identifier: None,
//         ae_invocation_identifier: None,
//     };

//     let socket_address = format!("{host}:{port}");
//     let mut tpkt_connection_factory = RustyTpktClientConnectionFactory::<TcpTpktConnection, TcpTpktReader, TcpTpktWriter>::new(socket_address.parse()?);
//     let mut factory = RustyMmsServiceFactory::new(data_pump);
//     factory.create_client_connection(&mut tpkt_connection_factory, mms_parameters).await?;

//     Ok(())
// }

// TODO: This may also close the association is the listener has no receivers.
async fn convert_object_identifier_reported(state: &str, reason: &str, association_state: Arc<RwLock<InitiatorIccpAssociationState>>, object_identifier: &ObjectIdentifier) -> Result<Oid<'static>, anyhow::Error> {
    let ae_title = match convert_object_identifiers(&object_identifier) {
        Ok(x) => x,
        Err(e) => {
            association_state.read().await.listener.send(IccpManagerEvent::IccpAssociationStateUpdate(state.into(), reason.into()));
            return Err(e);
        }
    };
    Ok(ae_title.to_owned())
}

fn convert_object_identifiers(object_identifier: &ObjectIdentifier) -> Result<Oid<'static>, anyhow::Error> {
    let ap_title_vec: Vec<u8> = object_identifier.into();
    let ap_title_oid: Oid<'_> = Any::from_tag_and_data(Tag::Oid, ap_title_vec.as_ref()).try_into()?;
    Ok(ap_title_oid.to_owned())
}

// TODO: This may also close the association is the listener has no receivers.
async fn convert_bigdecimal_to_bigint_reported(state: &str, reason: &str, association_state: Arc<RwLock<InitiatorIccpAssociationState>>, decimal: &BigDecimal) -> Result<BigInt, anyhow::Error> {
    let ae_qualifier = match convert_bigdecimal_to_bigint(&decimal) {
        Ok(x) => x,
        Err(e) => {
            association_state.read().await.listener.send(IccpManagerEvent::IccpAssociationStateUpdate(state.into(), reason.into()));
            return Err(e);
        }
    };
    Ok(ae_qualifier)
}

fn convert_bigdecimal_to_bigint(decimal: &BigDecimal) -> Result<BigInt, anyhow::Error> {
    let (ae_scaled_int, applied_exponent) = decimal.as_bigint_and_exponent();
    if applied_exponent > 0 {
        return Err(anyhow::anyhow!("AE Qualifier cannot be 0"));
    }
    match applied_exponent {
        _ if applied_exponent == 0 => Ok(ae_scaled_int),
        _ if applied_exponent < i32::MIN as i64 => Err(anyhow::anyhow!("AE Qualifier is too large")),
        _ if applied_exponent < 0 => Ok(ae_scaled_int.pow((-1 * applied_exponent) as u32)),
        _ => return Err(anyhow::anyhow!("AE Qualifier cannot have a decimal component")),
    }
}
