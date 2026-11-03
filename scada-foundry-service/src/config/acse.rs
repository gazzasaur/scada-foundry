use std::str::FromStr;

use der_parser::Oid;
use num_bigint::BigInt;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::serde_as;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcseListener {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcseConfiguration {
    pub acse_listeners: Vec<AcseListener>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcseAeTitle {
    #[serde(serialize_with = "serialise_oid", deserialize_with = "deserialise_oid")]
    pub ap_title: Oid<'static>,

    #[serde(serialize_with = "serialise_bigint", deserialize_with = "deserialise_bigint")]
    pub ae_qualifier: BigInt,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcseIdentityParameters {
    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tsap_id: Option<Vec<u8>>,

    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_selector: Option<Vec<u8>>,

    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_selector: Option<Vec<u8>>,

    pub ae_title: AcseAeTitle,

    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ap_invocation_identifier: Option<Vec<u8>>,
    
    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ae_invocation_identifier: Option<Vec<u8>>,
}

fn serialise_oid<S: Serializer>(value: &Oid<'static>, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_id_string())
}

fn deserialise_oid<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Oid<'static>, D::Error> {
    let value = String::deserialize(deserializer)?;
    let oid_value = Oid::from_str(value.as_str()).map_err(|_| D::Error::custom(format!("Invalid Oid String {}", value)))?;
    Ok(oid_value)
}

pub fn serialise_bigint<S: Serializer>(val: &BigInt, serializer: S) -> Result<S::Ok, S::Error> {
    serde_json::Number::from_str(&val.to_string()).map_err(serde::ser::Error::custom)?.serialize(serializer)
}

fn deserialise_bigint<'de, D: Deserializer<'de>>(deserializer: D) -> Result<BigInt, D::Error> {
    let value = serde_json::Number::deserialize(deserializer)?;
    let bigint_value = BigInt::from_str(value.as_str()).map_err(|_| D::Error::custom(format!("Invalid BigInt String {}", value)))?;
    Ok(bigint_value)
}
