use ::serde::de::*;
use std::str::FromStr;
use time::{OffsetDateTime, PrimitiveDateTime};

/// Deserialize a `time::PrimitiveDateTime`
pub fn prim_date_time<'de, D>(deser: D) -> Result<PrimitiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    // TODO borrow this
    let s = String::deserialize(deser)?;
    time::parse(&s, "%FT%TZ").map_err(Error::custom)
}

/// Deserialize a `time::OffsetDateTime` with an assumed ***UTC*** offset
pub fn assume_utc_date_time<'de, D>(deser: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    // TODO borrow this
    let s = String::deserialize(deser)? + " +0000";
    time::parse(&s, "%FT%TZ %z").map_err(Error::custom)
}

/// Deserialize using a `FromStr` impl
pub fn from_str<'de, D, T>(deser: D) -> Result<T, D::Error>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    // TODO borrow this
    let s = String::deserialize(deser)?;
    s.parse().map_err(Error::custom)
}
