use std::str::FromStr;

use hex::ToHex;
use num_bigint::BigInt;
use oid::ObjectIdentifier;
use serde::{Deserialize, Serialize};

use crate::{error::ScadaFoundryError, iccp::api::{IccpAeTitle, IccpAssociation, IccpAssociationType, IccpDataCenterParameters}};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebServiceIccpAeTitle {
    pub ap_title: ObjectIdentifier,
    pub ae_qualifier: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebServiceIccpDataCenterParameters {
    pub ae_title: WebServiceIccpAeTitle,
    pub tsap: String,
    pub ssap: String,
    pub psap: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebServiceIccpAssociation {
    pub id: String,
    pub name: String,
    pub association_type: IccpAssociationType,

    pub domain: String,
    pub bilateral_table: String,

    pub host: String,
    pub port: u16,
    pub local_data_center_parameters: WebServiceIccpDataCenterParameters,
    pub remote_data_center_parameters: WebServiceIccpDataCenterParameters,
}

impl From<&IccpAssociation> for WebServiceIccpAssociation {
    fn from(value: &IccpAssociation) -> Self {
        value.clone().into()
    }
}

impl From<IccpAssociation> for WebServiceIccpAssociation {
    fn from(value: IccpAssociation) -> Self {
        Self {
            id: value.id,
            name: value.name,
            association_type: value.association_type,
            domain: value.domain,
            bilateral_table: value.bilateral_table,
            host: value.host,
            port: value.port,
            local_data_center_parameters: WebServiceIccpDataCenterParameters {
                ae_title: WebServiceIccpAeTitle { ap_title: value.local_data_center_parameters.ae_title.ap_title, ae_qualifier: value.local_data_center_parameters.ae_title.ae_qualifier.to_string() },
                tsap: value.local_data_center_parameters.tsap.encode_hex_upper(),
                ssap: value.local_data_center_parameters.ssap.encode_hex_upper(),
                psap: value.local_data_center_parameters.psap.encode_hex_upper(),
            },
            remote_data_center_parameters: WebServiceIccpDataCenterParameters {
                ae_title: WebServiceIccpAeTitle { ap_title: value.remote_data_center_parameters.ae_title.ap_title, ae_qualifier: value.local_data_center_parameters.ae_title.ae_qualifier.to_string() },
                tsap: value.remote_data_center_parameters.tsap.encode_hex_upper(),
                ssap: value.remote_data_center_parameters.ssap.encode_hex_upper(),
                psap: value.remote_data_center_parameters.psap.encode_hex_upper(),
            },
        }
    }
}

impl TryFrom<&WebServiceIccpAssociation> for IccpAssociation {
    type Error = ScadaFoundryError;

    fn try_from(value: &WebServiceIccpAssociation) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl TryFrom<WebServiceIccpAssociation> for IccpAssociation {
    type Error = ScadaFoundryError;

    fn try_from(value: WebServiceIccpAssociation) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            name: value.name,
            association_type: value.association_type,
            domain: value.domain,
            bilateral_table: value.bilateral_table,
            host: value.host,
            port: value.port,
            local_data_center_parameters: IccpDataCenterParameters {
                ae_title: IccpAeTitle {
                    ap_title: value.local_data_center_parameters.ae_title.ap_title,
                    ae_qualifier: BigInt::from_str(value.local_data_center_parameters.ae_title.ae_qualifier.as_str()).unwrap(),
                },
                tsap: hex::decode(value.local_data_center_parameters.tsap).unwrap(),
                ssap: hex::decode(value.local_data_center_parameters.ssap).unwrap(),
                psap: hex::decode(value.local_data_center_parameters.psap).unwrap(),
            },
            remote_data_center_parameters: IccpDataCenterParameters {
                ae_title: IccpAeTitle {
                    ap_title: value.remote_data_center_parameters.ae_title.ap_title,
                    ae_qualifier: BigInt::from_str(value.remote_data_center_parameters.ae_title.ae_qualifier.as_str()).unwrap(),
                },
                tsap: hex::decode(value.remote_data_center_parameters.tsap).unwrap(),
                ssap: hex::decode(value.remote_data_center_parameters.ssap).unwrap(),
                psap: hex::decode(value.remote_data_center_parameters.psap).unwrap(),
            },
        })
    }
}