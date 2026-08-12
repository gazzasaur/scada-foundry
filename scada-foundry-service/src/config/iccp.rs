use rusty_iccp::{CurrentSourceValue, IccpData, NormalValue, TimestampQualityValue, ValidityValue};
use serde::{Deserialize, Serialize};

use crate::iccp::api::{IccpAssociation, IccpDataPointName};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IccpDataPointType {
    #[serde(rename_all = "camelCase")]
    RealQ { initial_value: f32 },
}

impl From<&IccpDataPointType> for IccpData {
    fn from(value: &IccpDataPointType) -> Self {
        match value {
            IccpDataPointType::RealQ { initial_value } => IccpData::RealQ(*initial_value, ValidityValue::Valid, CurrentSourceValue::Telemetered, NormalValue::Normal, TimestampQualityValue::Valid),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpDataPointSpecification {
    pub association_id: String,
    pub data_point_name: IccpDataPointName,
    pub data_point_type: IccpDataPointType,

    pub allow_write: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpTransferSetSpecification {
    pub name: String,
    pub domain: String,
    pub association_id: String,
    pub data_points: IccpDataPointName,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IccpConfiguration {
    pub associations: Vec<IccpAssociation>,
    pub data_points: Vec<IccpDataPointSpecification>,
    pub transfer_sets: Vec<IccpDataPointSpecification>,
}
