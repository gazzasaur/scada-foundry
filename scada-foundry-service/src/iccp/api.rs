use crate::iccp::converter::deserialise_ae_title;
use crate::iccp::converter::serialise_bigint;
use num_bigint::BigInt;
use oid::ObjectIdentifier;
use rusty_iccp::IccpData;
use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum IccpDataPointName {
    Vcc(String),
    Icc(String, String),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct IccpDataPointKey {
    association_id: String,
    point_name: IccpDataPointName,
}

pub struct IccpDataPointValue {
    pub association_id: String,
    pub name: Option<String>,
    pub point_name: IccpDataPointKey,
    pub value: IccpData,

    pub source: String,         // The unique identifier of the source into this system
    pub recorded: u64,          // Recorded in this system
    pub updated: u64,           // Updated on this node
    pub timestamp: Option<u64>, // The timestamp received by SCADA

    pub allow_write: bool,
}

pub enum IccpAssociationState {
    Idle,
    Failed,
    Connected,
}

pub struct IccpAssociationStatus {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub status_description: String,
    pub state: IccpAssociationState,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IccpAssociationType {
    ClientUnidirectional,
    ServerUnidirectional,
    ClientBidirectional,
    ServerBidirectional,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpAeTitle {
    pub ap_title: ObjectIdentifier,

    #[serde(serialize_with = "serialise_bigint")]
    pub ae_qualifier: BigInt,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpDataCenterParameters {
    #[serde(deserialize_with = "deserialise_ae_title")]
    pub ae_title: IccpAeTitle,

    #[serde(with = "hex")]
    pub tsap: Vec<u8>,

    #[serde(with = "hex")]
    pub ssap: Vec<u8>,

    #[serde(with = "hex")]
    pub psap: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpAssociation {
    pub id: String,
    pub name: String,
    pub association_type: IccpAssociationType,

    pub domain: String,
    pub bilateral_table: String,

    pub host: String,
    pub port: u16,
    pub local_data_center_parameters: IccpDataCenterParameters,
    pub remote_data_center_parameters: IccpDataCenterParameters,
}
