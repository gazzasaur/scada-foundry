use std::sync::{Arc, atomic::AtomicBool};

use bigdecimal::BigDecimal;
use der_parser::{Oid, asn1_rs::Any, der::Tag};
use futures::{StreamExt, stream::FuturesUnordered};
use num_bigint::BigInt;
use oid::ObjectIdentifier;
use rusty_mms_service::{MmsServiceConnectionIdentityParameters, MmsServiceConnectionParameters, RustyMmsServiceFactory, RustyTpktClientConnectionFactory, datapump::MmsServiceDataPump};
use rusty_tpkt::{TcpTpktConnection, TcpTpktReader, TcpTpktWriter};
use tokio::sync::
    Mutex
;
use tokio::sync::{RwLock, mpsc::UnboundedSender};
use tracing::error;

use crate::config::iccp::InitiatorIccpAssociation;

enum IccpWorkerRequest {
    CreateInitiator { listeners: Arc<RwLock<Vec<UnboundedSender<IccpEvent>>>>, association: InitiatorIccpAssociation },
    Close,
}

pub enum IccpAssociationState {
    Pending,
    Connecting,
    Listening,
    Connected,
    Failed { cause: String, retry_timestamp: u64 },
}

pub enum IccpEvent {
    AssociationStateUpdate { uuid: String, state: IccpAssociationState },
}

#[derive(Clone)]
pub struct IccpManager {
    worker: Arc<IccpManagerWorker>,
}

impl IccpManager {
    pub async fn new() -> Self {
        IccpManager { worker: Arc::new(IccpManagerWorker::new().await) }
    }

    pub async fn serve(&self) {
        self.worker.process().await;
        //     let al = self.listeners.clone();
        //     let mut all_tasks = FuturesUnordered::new();
        //     let (request_sender, mut request_receiver): (UnboundedSender<IccpWorkerRequest>, UnboundedReceiver<IccpWorkerRequest>) = mpsc::unbounded_channel();

        //     loop {
        //         match tokio::time::timeout(Duration::from_millis(100), request_receiver.recv()).await {
        //             Err(_) => all_tasks.push(Box::pin(async {
        //                 let data_pump = Arc::new(MmsServiceDataPump::new(Arc::new(AtomicBool::new(true)), Arc::new(Mutex::new(Vec::new()))));
        //                 iccp_initiator_connect(
        //                     data_pump,
        //                     InitiatorIccpAssociation {
        //                         uuid: "".into(),
        //                         name: "".into(),
        //                         role: crate::config::iccp::InitiatorRole::Client,
        //                         authentication: crate::config::iccp::InitiatorAuthenticationScheme::None,
        //                         local_control_center: IccpInitiatorControlCenterInformation {
        //                             tsap_address: vec![],
        //                             ssap_address: vec![],
        //                             psap_address: vec![],
        //                             ae_title: AeTitle { ap_title: ObjectIdentifier::try_from("1.2.3.4").unwrap(), ae_qualifier: BigDecimal::from(1) },
        //                         },
        //                         remote_control_center: IccpResponderControlCenterInformation {
        //                             host: "".into(),
        //                             port: 102,
        //                             tsap_address: vec![],
        //                             ssap_address: vec![],
        //                             psap_address: vec![],
        //                             ae_title: AeTitle { ap_title: ObjectIdentifier::try_from("1.2.3.4").unwrap(), ae_qualifier: BigDecimal::from(1) },
        //                         },
        //                         data_sets: vec![],
        //                     },
        //                 )
        //                 .await;
        //             })),
        //             Ok(_) => (),
        //         };
        //         println!("Looping4");
        //         all_tasks.next().await;
        //     }
    }
}

enum InitiatorAssociationState {
    New(InitiatorIccpAssociation),
    // Connecting(InitiatorIccpAssociation, UnboundedReceiver<Result<RustyIccpClient, Box<dyn Error>>>),
    // Connected(InitiatorIccpAssociation, RustyIccpClient),
}

struct IccpManagerWorker {
    data_pump: Arc<MmsServiceDataPump>,
}

impl IccpManagerWorker {
    async fn new() -> Self {
        let data_pump = Arc::new(MmsServiceDataPump::new(Arc::new(AtomicBool::new(true)), Arc::new(Mutex::new(Vec::new()))));
        IccpManagerWorker { data_pump }
    }

    async fn process(&mut self) {
        let initiator_associations: Arc<Mutex<Vec<InitiatorAssociationState>>> = Arc::new(Mutex::new(Vec::new()));
        let mut current_bindings = FuturesUnordered::new();

        loop {
            // match self.configure.try_recv() {
            //     Ok(IccpWorkerRequest::Close) => return,
            //     Ok(IccpWorkerRequest::CreateInitiator { listeners, association }) => {
            //         initiator_associations.lock().await.push(InitiatorAssociationState::New(association));
            //     }

            //     Err(TryRecvError::Empty) => (),
            //     Err(TryRecvError::Disconnected) => return,
            // }

            let fgh = {
                let mut abc = initiator_associations.lock().await;
                let abcd: Vec<InitiatorAssociationState> = abc.drain(..).collect();
                abcd
            };

            for initiator_association in fgh {
                match initiator_association {
                    InitiatorAssociationState::New(initiator_iccp_association) => {
                        let t1 = iccp_initiator_connect(self.data_pump.clone(), initiator_iccp_association.clone());
                        current_bindings.push(t1);
                        current_bindings.next().await;
                        // iccp_initiator_connect(dp, config).await;
                    } // InitiatorAssociationState::Connecting(initiator_iccp_association) => todo!(),
                      // InitiatorAssociationState::Connected(initiator_iccp_association, rusty_iccp_client) => todo!(),
                }
            }
        }
    }
}

// TODO Allow arbitrary configuration of the datapump and connection mapping
async fn iccp_initiator_connect(data_pump: Arc<MmsServiceDataPump>, initiator_iccp_association: InitiatorIccpAssociation) {
    print!("Looping");
    if let Err(e) = try_iccp_initiator_connect(data_pump, initiator_iccp_association).await {
        error!("Failed to create connection: {}", e);
    }
}

async fn try_iccp_initiator_connect(data_pump: Arc<MmsServiceDataPump>, initiator_iccp_association: InitiatorIccpAssociation) -> Result<(), anyhow::Error> {
    let host = initiator_iccp_association.remote_control_center.host;
    let port = initiator_iccp_association.remote_control_center.port;

    let mut mms_parameters = MmsServiceConnectionParameters::default();
    mms_parameters.calling = MmsServiceConnectionIdentityParameters {
        tsap_id: Some(initiator_iccp_association.local_control_center.tsap_address),
        session_selector: Some(initiator_iccp_association.local_control_center.ssap_address),
        presentation_selector: Some(initiator_iccp_association.local_control_center.psap_address),
        ap_title: Some(convert_object_identifiers(&initiator_iccp_association.local_control_center.ae_title.ap_title)?),
        ae_qualifier: Some(convert_bigdecimal_to_bigint(&initiator_iccp_association.local_control_center.ae_title.ae_qualifier)?.to_signed_bytes_be()),
        ap_invocation_identifier: None,
        ae_invocation_identifier: None,
    };
    mms_parameters.called = MmsServiceConnectionIdentityParameters {
        tsap_id: Some(initiator_iccp_association.remote_control_center.tsap_address),
        session_selector: Some(initiator_iccp_association.remote_control_center.ssap_address),
        presentation_selector: Some(initiator_iccp_association.remote_control_center.psap_address),
        ap_title: Some(convert_object_identifiers(&initiator_iccp_association.remote_control_center.ae_title.ap_title)?),
        ae_qualifier: Some(convert_bigdecimal_to_bigint(&initiator_iccp_association.remote_control_center.ae_title.ae_qualifier)?.to_signed_bytes_be()),
        ap_invocation_identifier: None,
        ae_invocation_identifier: None,
    };

    let socket_address = format!("{host}:{port}");
    let mut tpkt_connection_factory = RustyTpktClientConnectionFactory::<TcpTpktConnection, TcpTpktReader, TcpTpktWriter>::new(socket_address.parse()?);
    let mut factory = RustyMmsServiceFactory::new(data_pump);
    factory.create_client_connection(&mut tpkt_connection_factory, mms_parameters).await?;

    Ok(())
}

fn convert_object_identifiers(object_identifier: &ObjectIdentifier) -> Result<Oid<'static>, anyhow::Error> {
    let ap_title_vec: Vec<u8> = object_identifier.into();
    let ap_title_oid: Oid<'_> = Any::from_tag_and_data(Tag::Oid, ap_title_vec.as_ref()).try_into()?;
    Ok(ap_title_oid.to_owned())
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
