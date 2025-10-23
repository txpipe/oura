use bech32::FromBase32;
use serde::de::{DeserializeOwned, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

use super::PatternOf;

struct StringOrStructVisitor<T>(PhantomData<T>)
where
    T: DeserializeOwned + FromStr;

impl<'de, T> Visitor<'de> for StringOrStructVisitor<T>
where
    T: DeserializeOwned + FromStr<Err = anyhow::Error>,
{
    type Value = StringOrStruct<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string or map")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let inner = T::from_str(value).map_err(serde::de::Error::custom)?;
        Ok(StringOrStruct(inner))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let inner = Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
        Ok(StringOrStruct(inner))
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(transparent)]
pub struct StringOrStruct<T>(pub T);

impl<'de, T> Deserialize<'de> for StringOrStruct<T>
where
    T: DeserializeOwned + FromStr<Err = anyhow::Error>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StringOrStructVisitor(PhantomData))
    }
}

impl<T> StringOrStruct<T> {
    pub fn unwrap(self) -> T {
        self.0
    }
}

impl<T> Deref for StringOrStruct<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromStr for StringOrStruct<T>
where
    T: FromStr<Err = anyhow::Error>,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = T::from_str(s)?;
        Ok(Self(inner))
    }
}

impl<S, T> PatternOf<S> for StringOrStruct<T>
where
    T: PatternOf<S>,
{
    fn is_match(&self, subject: S) -> super::MatchOutcome {
        self.0.is_match(subject)
    }
}

impl<T> From<T> for StringOrStruct<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

pub trait FromBech32: Sized {
    fn from_bech32_parts(hrp: &str, content: Vec<u8>) -> Option<Self>;

    fn from_bech32(s: &str) -> anyhow::Result<Self> {
        let (hrp, content, _) = bech32::decode(s)?;
        let content = Vec::<u8>::from_base32(&content)?;

        Self::from_bech32_parts(&hrp, content)
            .ok_or_else(|| anyhow::anyhow!("bech32 hrp '{}' is not compatible for this type", hrp))
    }
}

pub mod regex_pattern {
    use regex::Regex;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(regex: &Regex, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(regex.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Regex::new(&s).map_err(serde::de::Error::custom)
    }
}
