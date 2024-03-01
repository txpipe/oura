use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct FlexBytes(Vec<u8>);

impl Deref for FlexBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<u8>> for FlexBytes {
    fn from(value: Vec<u8>) -> Self {
        FlexBytes(value)
    }
}

impl From<&str> for FlexBytes {
    fn from(value: &str) -> Self {
        FlexBytes(value.as_bytes().to_vec())
    }
}

impl Serialize for FlexBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = hex::encode(&self.0);
        serializer.serialize_str(&hex_string)
    }
}

struct FlexBytesVisitor;

impl<'de> Visitor<'de> for FlexBytesVisitor {
    type Value = FlexBytes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a hex or bech32 string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let bytes = hex::decode(value).map_err(de::Error::custom)?;
        Ok(FlexBytes(bytes))
    }
}

impl<'de> Deserialize<'de> for FlexBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FlexBytesVisitor)
    }
}
