use core::fmt;
use std::{ops::Deref, str::FromStr};

use log::info;
use pallas::ouroboros::network::{
    chainsync::TipFinder,
    handshake::{MAINNET_MAGIC, TESTNET_MAGIC},
    machines::{primitives::Point, run_agent},
    multiplexer::Channel,
};
use serde::{de::Visitor, Deserializer};
use serde_derive::Deserialize;

use crate::framework::Error;

#[derive(Debug, Deserialize)]
pub enum BearerKind {
    Tcp,
    Unix,
}

#[derive(Debug, Deserialize)]
pub struct AddressArg(pub BearerKind, pub String);

impl FromStr for BearerKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unix" => Ok(BearerKind::Unix),
            "tcp" => Ok(BearerKind::Tcp),
            _ => Err("can't parse bearer type value"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MagicArg(u64);

impl Deref for MagicArg {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MagicArg {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = match s {
            "testnet" => MagicArg(TESTNET_MAGIC),
            "mainnet" => MagicArg(MAINNET_MAGIC),
            _ => MagicArg(u64::from_str(s).map_err(|_| "can't parse magic value")?),
        };

        Ok(m)
    }
}

pub fn deserialize_magic_arg<'de, D>(deserializer: D) -> Result<Option<MagicArg>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MagicArgVisitor;

    impl<'de> Visitor<'de> for MagicArgVisitor {
        type Value = Option<MagicArg>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<MagicArg>, E>
        where
            E: serde::de::Error,
        {
            let value = FromStr::from_str(value).map_err(serde::de::Error::custom)?;
            Ok(Some(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<MagicArg>, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(MagicArg(value)))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(MagicArgVisitor)
}

pub fn get_wellknonwn_chain_point(magic: u64) -> Result<Point, Error> {
    match magic {
        MAINNET_MAGIC => Ok(Point(
            4492799,
            hex::decode("f8084c61b6a238acec985b59310b6ecec49c0ab8352249afd7268da5cff2a457")?,
        )),
        TESTNET_MAGIC => Ok(Point(
            1598399,
            hex::decode("7e16781b40ebf8b6da18f7b5e8ade855d6738095ef2f1c58c77e88b6e45997a4")?,
        )),
        _ => Err("don't have a well-known chain point for the requested magic".into()),
    }
}

pub fn find_end_of_chain(
    channel: &mut Channel,
    wellknown_point: Point,
) -> Result<Point, crate::framework::Error> {
    let agent = TipFinder::initial(wellknown_point);
    let agent = run_agent(agent, channel)?;
    info!("chain point query output: {:?}", agent.output);

    match agent.output {
        Some(tip) => Ok(tip.0),
        None => Err("failure acquiring end of chain".into()),
    }
}
