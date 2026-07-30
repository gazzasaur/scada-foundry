use bigdecimal::BigDecimal;
use der_parser::{Oid, asn1_rs::Any, der::Tag};
use num_bigint::BigInt;
use oid::ObjectIdentifier;

fn convert_object_identifiers(object_identifier: &ObjectIdentifier) -> Result<Oid<'static>, anyhow::Error> {
    let ap_title_vec: Vec<u8> = object_identifier.into();
    let ap_title_oid: Oid<'_> = Any::from_tag_and_data(Tag::Oid, ap_title_vec.as_ref()).try_into()?;
    Ok(ap_title_oid.to_owned())
}

fn convert_bigdecimal_to_bigint(decimal: &BigDecimal) -> Result<BigInt, anyhow::Error> {
    let (ae_scaled_int, applied_exponent) = decimal.as_bigint_and_exponent();
    if applied_exponent > 0 {
        return Err(anyhow::anyhow!("AE Qualifier cannot be 0"));
    }
    match applied_exponent {
        _ if applied_exponent == 0 => Ok(ae_scaled_int),
        _ if applied_exponent < i32::MIN as i64 => Err(anyhow::anyhow!("AE Qualifier is too large")),
        _ if applied_exponent < 0 => Ok(ae_scaled_int.pow((-1 * applied_exponent) as u32)),
        _ => return Err(anyhow::anyhow!("AE Qualifier cannot have a decimal component")),
    }
}
