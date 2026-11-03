use crate::config::acse::AcseIdentityParameters;
use rusty_iccp::IccpData;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IccpDataPointName {
    Vcc { name: String },
    Icc { domain: String, name: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IccpDataPointKey {
    pub association_id: String,
    pub point_name: IccpDataPointName,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IccpDataPointValue {
    pub association_id: String,
    pub data_point_name: IccpDataPointName,

    pub value: IccpData,

    pub source: String,         // The unique identifier of the source into this system
    pub recorded: u64,          // Recorded in this system
    pub updated: u64,           // Updated on this node
    pub timestamp: Option<u64>, // The timestamp received by SCADA

    pub allow_write: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IccpAssociationListener {
    id: String,
    name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum IccpAssociationSource {
    Calling {
        host: String,
        port: u16,
    },
    Called {
        #[serde(skip_serializing_if = "Option::is_none")]
        listener: Option<IccpAssociationListener>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IccpAssociation {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub version: String,

    pub local_domain: String,
    pub remote_domain: String,
    pub local_bilateral_table: String,
    pub remote_bilateral_table: String,

    pub source: IccpAssociationSource,

    pub local_data_center_parameters: AcseIdentityParameters,
    pub remote_data_center_parameters: AcseIdentityParameters,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum IccpAssociationStatus {
    Idle,
    Failed,
    Connected,
    Connecting,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IccpServerOperationalAssociation {
    pub state: String,
    pub status: String,
    pub association: IccpAssociation,
}
