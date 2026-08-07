use core::fmt;
use std::str::FromStr;

use der_parser::{Oid, asn1_rs::Any, der::Tag};
use num_bigint::BigInt;
use oid::ObjectIdentifier;
use serde::{Deserializer, Serialize, Serializer, de::Visitor};
use serde_json::value::RawValue;

use crate::{error::ScadaFoundryError, iccp::api::IccpAeTitle};

pub fn convert_object_identifiers(object_identifier: &ObjectIdentifier) -> Result<Oid<'static>, anyhow::Error> {
    let ap_title_vec: Vec<u8> = object_identifier.into();
    let ap_title_oid: Oid<'_> = Any::from_tag_and_data(Tag::Oid, ap_title_vec.as_ref()).try_into()?;
    Ok(ap_title_oid.to_owned())
}

pub fn serialise_bigint<S: Serializer>(val: &BigInt, serializer: S) -> Result<S::Ok, S::Error> {
    serde_json::Number::from_str(&val.to_string()).map_err(serde::ser::Error::custom)?.serialize(serializer)
}

struct AeTitleVisitor;

impl<'de> Visitor<'de> for AeTitleVisitor {
    type Value = IccpAeTitle;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> core::fmt::Result {
        write!(formatter, "an application entity title")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut ap_title_string = None;
        let mut ae_qualifier_string = None;
        let (ap_title, ae_qualifier) = loop {
            match (map.next_entry::<&'de str, &'de RawValue>(), &ap_title_string, &ae_qualifier_string) {
                (Err(e), _, _) => return Err(serde::de::Error::custom(format!("{e}"))),
                (Ok(None), None, _) => return Err(serde::de::Error::custom(format!("apTitle must be defined on an AE Title"))),
                (Ok(None), _, None) => return Err(serde::de::Error::custom(format!("aeQualifier must be defined on an AE Title"))),
                (Ok(None), Some(ap_title), Some(ae_qualifier)) => break (ap_title, ae_qualifier),

                (Ok(Some(("apTitle", _))), Some(_), _) => return Err(serde::de::Error::custom(format!("apTitle on AE Title can only be declared once"))),
                (Ok(Some(("aeQualifier", _))), _, Some(_)) => return Err(serde::de::Error::custom(format!("aeQualifier on AE Title aeQualifier can only be declared once"))),

                (Ok(Some(("apTitle", value))), None, _) => ap_title_string = Some(value),
                (Ok(Some(("aeQualifier", value))), _, None) => ae_qualifier_string = Some(value),
                (Ok(Some((unknown_key, _))), _, _) => return Err(serde::de::Error::custom(format!("Unknown field on AE Title: {unknown_key}"))),
            }
        };

        let mut ae_qualifier_string = ae_qualifier.to_string();
        if ae_qualifier_string.contains("\"") {
            ae_qualifier_string = serde_json::from_str::<String>(&ae_qualifier_string.as_str()).map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        }

        Ok(IccpAeTitle {
            ap_title: ObjectIdentifier::try_from(serde_json::from_str::<String>(&ap_title.to_string()).map_err(|e| serde::de::Error::custom(format!("Failed to parse ApTitle on Ae Title: {e:?}")))?)
                .map_err(|e| serde::de::Error::custom(format!("Failed to parse ApTitle on Ae Title: {e:?}")))?,
            ae_qualifier: BigInt::from_str(ae_qualifier.to_string().as_str()).map_err(|e| serde::de::Error::custom(format!("{e:?}")))?,
        })
    }
}

pub fn deserialise_ae_title<'de, D: Deserializer<'de>>(deserialiser: D) -> Result<IccpAeTitle, D::Error> {
    deserialiser.deserialize_map(AeTitleVisitor)
}
