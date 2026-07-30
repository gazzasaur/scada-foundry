use bigdecimal::BigDecimal;
use oid::ObjectIdentifier;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IccpAssociationType {
    ClientUnidirectional,
    ClientBidirectional,
    ServerUnidirectional,
    ServerBidirectional,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IccpDataPointType {
    RealQ,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpDataPointSpecification {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub data_point_type: IccpDataPointType,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AeTitle {
    pub ap_title: ObjectIdentifier,

    #[serde(with = "bigdecimal::serde::json_num")]
    pub ae_qualifier: BigDecimal,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpDataCenterSetSpecification {
    pub ae_title: AeTitle,

    #[serde(with = "hex")]
    pub tsap_address: Vec<u8>,

    #[serde(with = "hex")]
    pub ssap_address: Vec<u8>,

    #[serde(with = "hex")]
    pub psap_address: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpTransferSetSpecification {}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpAssociation {
    pub id: String,
    pub name: String,

    pub connection_domain: String,
    pub connection_bilateral_tablw: String,

    pub association_type: IccpAssociationType,

    pub host: String,
    pub port: u16,

    pub local_data_center: IccpDataCenterSetSpecification,
    pub remote_data_center: IccpDataCenterSetSpecification,

    pub local_data_points: Vec<IccpDataPointSpecification>,
    pub remote_data_points: Vec<IccpDataPointSpecification>,

    pub transfer_sets: Vec<IccpTransferSetSpecification>,
    // TODO Devices
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpConfiguration {
    pub associations: Vec<IccpAssociation>,
}
