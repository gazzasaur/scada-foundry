use serde::{Deserialize, Serialize};

use crate::iccp::api::{IccpAeTitle, IccpAssociation};

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

// #[derive(Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct AeTitle {
//     pub ap_title: ObjectIdentifier,

//     #[serde(with = "bigdecimal::serde::json_num")]
//     pub ae_qualifier: BigDecimal,
// }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpDataCenterSetSpecification {
    pub ae_title: IccpAeTitle,

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
pub struct IccpConfiguration {
    pub associations: Vec<IccpAssociation>,
}
