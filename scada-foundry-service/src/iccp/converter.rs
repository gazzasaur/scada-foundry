use core::fmt;
use std::str::FromStr;

use der_parser::{Oid, asn1_rs::Any, der::Tag};
use num_bigint::BigInt;
use oid::ObjectIdentifier;
use serde::{Deserializer, Serialize, Serializer, de::Visitor};

pub fn convert_object_identifiers(object_identifier: &ObjectIdentifier) -> Result<Oid<'static>, anyhow::Error> {
    let ap_title_vec: Vec<u8> = object_identifier.into();
    let ap_title_oid: Oid<'_> = Any::from_tag_and_data(Tag::Oid, ap_title_vec.as_ref()).try_into()?;
    Ok(ap_title_oid.to_owned())
}

pub fn serialise_bigint<S: Serializer>(val: &BigInt, serializer: S) -> Result<S::Ok, S::Error> {
    serde_json::Number::from_str(&val.to_string()).map_err(serde::ser::Error::custom)?.serialize(serializer)
}

struct BigIntVisitor;

impl<'de> Visitor<'de> for BigIntVisitor {
    type Value = BigInt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> core::fmt::Result {
        println!("HERE2");
        write!(formatter, "a number")
    }

    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<BigInt, E> {
        println!("HERE3");
        BigInt::from_str(value.as_str()).map_err(|err| E::custom(format!("{}", err)))
    }
}

pub fn deserialise_bigint<'de, D: Deserializer<'de>>(deserialiser: D) -> Result<BigInt, D::Error> {
    println!("HERE1");
    deserialiser.deserialize_string(BigIntVisitor)
}
