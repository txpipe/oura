use core::fmt;
use std::{ops::Deref, str::FromStr};

use log::info;
use pallas::network::{
    miniprotocols::{chainsync::TipFinder, run_agent, Point, MAINNET_MAGIC, TESTNET_MAGIC},
    multiplexer::Channel,
};
use serde::{de::Visitor, Deserializer};
use serde::{Deserialize, Serialize};

use crate::{
    utils::{ChainWellKnownInfo, Utils},
    Error,
};

#[derive(Debug, Deserialize)]
pub enum BearerKind {
    Tcp,
    #[cfg(target_family = "unix")]
    Unix,
}

#[derive(Debug, Deserialize)]
pub struct AddressArg(pub BearerKind, pub String);

impl FromStr for BearerKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tcp" => Ok(BearerKind::Tcp),
            #[cfg(target_family = "unix")]
            "unix" => Ok(BearerKind::Unix),
            _ => Err("can't parse bearer type value"),
        }
    }
}

/// A serialization-friendly chain Point struct using a hex-encoded hash
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointArg(pub u64, pub String);

impl TryInto<Point> for PointArg {
    type Error = Error;

    fn try_into(self) -> Result<Point, Self::Error> {
        let hash = hex::decode(&self.1)?;
        Ok(Point::Specific(self.0, hash))
    }
}

impl FromStr for PointArg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(',') {
            let mut parts: Vec<_> = s.split(',').collect();
            let slot = parts.remove(0).parse()?;
            let hash = parts.remove(0).to_owned();
            Ok(PointArg(slot, hash))
        } else {
            Err("Can't parse chain point value, expecting `slot,hex-hash` format".into())
        }
    }
}

impl ToString for PointArg {
    fn to_string(&self) -> String {
        format!("{},{}", self.0, self.1)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MagicArg(pub u64);

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

impl Default for MagicArg {
    fn default() -> Self {
        Self(MAINNET_MAGIC)
    }
}

pub(crate) fn deserialize_magic_arg<'de, D>(deserializer: D) -> Result<Option<MagicArg>, D::Error>
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

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum IntersectArg {
    Tip,
    Origin,
    Point(PointArg),
    Fallbacks(Vec<PointArg>),
}

pub(crate) fn find_end_of_chain(
    channel: &mut Channel,
    well_known: &ChainWellKnownInfo,
) -> Result<Point, crate::Error> {
    let point = Point::Specific(
        well_known.shelley_known_slot,
        hex::decode(&well_known.shelley_known_hash)?,
    );

    let agent = TipFinder::initial(point);
    let agent = run_agent(agent, channel)?;
    info!("chain point query output: {:?}", agent.output);

    match agent.output {
        Some(tip) => Ok(tip.0),
        None => Err("failure acquiring end of chain".into()),
    }
}

pub(crate) fn define_start_point(
    intersect: &Option<IntersectArg>,
    since: &Option<PointArg>,
    utils: &Utils,
    cs_channel: &mut Channel,
) -> Result<Option<Vec<Point>>, Error> {
    let cursor = utils.get_cursor_if_any();

    match cursor {
        Some(cursor) => {
            log::info!("found persisted cursor, will use as starting point");
            let points = vec![cursor.try_into()?];

            Ok(Some(points))
        }
        None => match intersect {
            Some(IntersectArg::Fallbacks(x)) => {
                log::info!("found 'fallbacks' intersect argument, will use as starting point");
                let points: Result<Vec<_>, _> = x.iter().map(|x| x.clone().try_into()).collect();

                Ok(Some(points?))
            }
            Some(IntersectArg::Origin) => {
                log::info!("found 'origin' instersect argument, will use as starting point");

                Ok(None)
            }
            Some(IntersectArg::Point(x)) => {
                log::info!("found 'point' intersect argument, will use as starting point");
                let points = vec![x.clone().try_into()?];

                Ok(Some(points))
            }
            Some(IntersectArg::Tip) => {
                log::info!("found 'tip' intersect argument, will use as starting point");
                let tip = find_end_of_chain(cs_channel, &utils.well_known)?;
                let points = vec![tip];

                Ok(Some(points))
            }
            None => match since {
                Some(x) => {
                    log::info!("explicit 'since' argument, will use as starting point");
                    log::warn!("`since` value is deprecated, please use `intersect`");
                    let points = vec![x.clone().try_into()?];

                    Ok(Some(points))
                }
                None => {
                    log::info!("no starting point specified, will use tip of chain");
                    let tip = find_end_of_chain(cs_channel, &utils.well_known)?;
                    let points = vec![tip];

                    Ok(Some(points))
                }
            },
        },
    }
}
