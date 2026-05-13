use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use oid::ObjectIdentifier;
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Serialize, Deserialize)]
pub struct IccpConfiguration {
    pub initator_associations: Vec<InitiatorIccpAssociation>,
    pub responder_associations: Vec<ResponderIccpAssociation>,
}

#[derive(Serialize, Deserialize)]
pub struct IccpDataSet {
    pub domain: String,
    pub name: String,

    pub points: Vec<IccpDataPoint>,
}

#[derive(Serialize, Deserialize)]
pub enum IccpPointName {
    App(String),
    Vcc(String),
    Icc(String, String),
}

#[derive(Serialize, Deserialize)]
pub enum IccpPointDataType {
    Real,
    RealQ,
    Discrete,
    DiscreteQ,
    State,
    StateQ,
}

#[derive(Serialize, Deserialize)]
pub struct IccpDataPoint {
    pub uuid: String,
    pub name: IccpPointName,
    pub data_type: IccpPointDataType,
}

#[derive(Serialize, Deserialize)]
pub struct InitiatorIccpAssociation {
    pub uuid: String,
    pub name: String,
    pub role: InitiatorRole,
    pub authentication: InitiatorAuthenticationScheme,
    pub local_control_center: IccpInitiatorControlCenterInformation,
    pub remote_control_center: IccpResponderControlCenterInformation,

    pub data_sets: Vec<IccpDataSet>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponderIccpAssociation {
    pub uuid: String,
    pub name: String,
    pub role: ResponderRole,
    pub authentication: ResponderAuthenticationScheme,
    pub local_matcher: LocalIccpControlCenterMatcher,
    pub remote_matcher: RemoteIccpControlCenterMatcher,

    pub points: Vec<IccpDataPoint>,
}

#[derive(Serialize, Deserialize)]
pub struct IccpInitiatorControlCenterInformation {
    #[serde(with = "hex")]
    pub tsap_address: Vec<u8>,

    #[serde(with = "hex")]
    pub ssap_address: Vec<u8>,

    #[serde(with = "hex")]
    pub psap_address: Vec<u8>,

    pub ae_title: AeTitle,
}

#[derive(Serialize, Deserialize)]
pub struct IccpResponderControlCenterInformation {
    pub host: String,
    pub port: u16,
    
    #[serde(with = "hex")]
    pub tsap_address: Vec<u8>,

    #[serde(with = "hex")]
    pub ssap_address: Vec<u8>,

    #[serde(with = "hex")]
    pub psap_address: Vec<u8>,

    pub ae_title: AeTitle,
}

#[derive(Serialize, Deserialize)]
pub enum InitiatorRole {
    Client
}

#[derive(Serialize, Deserialize)]
pub enum ResponderRole {
    Server
}

#[derive(Serialize, Deserialize)]
pub enum InitiatorAuthenticationScheme {
    None,
}

#[derive(Serialize, Deserialize)]
pub enum ResponderAuthenticationScheme {
    None,
}

#[derive(Serialize, Deserialize)]
pub struct AeTitle {
    pub ap_title: ObjectIdentifier,

    #[serde(with = "bigdecimal::serde::json_num")]
    pub ae_qualifier: BigDecimal,
}

#[derive(Serialize, Deserialize)]
pub enum LocalIccpControlCenterMatcher {
    Masqurade,
}

#[derive(Serialize, Deserialize)]
pub enum RemoteIccpControlCenterMatcher {
    Relaxed {
        tsap_address: SapAddressMatcher,
        ssap_address: SapAddressMatcher,
        psap_address: SapAddressMatcher,
        ae_title: AeTitleMatcher,
    },
}

#[derive(Serialize, Deserialize)]
pub enum SapAddressMatcher {
    Any,

    #[serde(with = "hex")]
    Exact(Vec<u8>),
}

#[derive(Serialize, Deserialize)]
pub enum AeTitleMatcher {
    ApTitleOnly(ObjectIdentifier),
}
