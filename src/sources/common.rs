use core::fmt;
use std::{ops::Deref, str::FromStr, time::Duration};

use pallas::{
    ledger::traverse::{probe, Era},
    network::{
        miniprotocols::{chainsync, Point, MAINNET_MAGIC, TESTNET_MAGIC},
        multiplexer::{bearers::Bearer, StdChannel, StdPlexer},
    },
};

use serde::{de::Visitor, Deserializer};
use serde::{Deserialize, Serialize};

use crate::{
    mapper::EventWriter,
    utils::{retry, SwallowResult, Utils},
    Error,
};

// TODO: these should come from Pallas
use crate::utils::{PREPROD_MAGIC, PREVIEW_MAGIC};

#[derive(Debug, Deserialize, Clone)]
pub enum BearerKind {
    Tcp,
    #[cfg(target_family = "unix")]
    Unix,
}

#[derive(Debug, Deserialize, Clone)]
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
            "preview" => MagicArg(PREVIEW_MAGIC),
            "preprod" => MagicArg(PREPROD_MAGIC),
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
            formatter.write_str("string or number")
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

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(MagicArg(v as u64)))
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

#[derive(Deserialize, Debug, Clone)]
pub struct RetryPolicy {
    #[serde(default = "RetryPolicy::default_max_retries")]
    pub chainsync_max_retries: u32,

    #[serde(default = "RetryPolicy::default_max_backoff")]
    pub chainsync_max_backoff: u32,

    #[serde(default = "RetryPolicy::default_max_retries")]
    pub connection_max_retries: u32,

    #[serde(default = "RetryPolicy::default_max_backoff")]
    pub connection_max_backoff: u32,
}

impl RetryPolicy {
    fn default_max_retries() -> u32 {
        50
    }

    fn default_max_backoff() -> u32 {
        60
    }
}

pub fn setup_multiplexer_attempt(bearer: &BearerKind, address: &str) -> Result<StdPlexer, Error> {
    match bearer {
        BearerKind::Tcp => {
            let bearer = Bearer::connect_tcp(address)?;
            Ok(StdPlexer::new(bearer))
        }
        #[cfg(target_family = "unix")]
        BearerKind::Unix => {
            let unix = Bearer::connect_unix(address)?;
            Ok(StdPlexer::new(unix))
        }
    }
}

pub fn setup_multiplexer(
    bearer: &BearerKind,
    address: &str,
    retry: &Option<RetryPolicy>,
) -> Result<StdPlexer, Error> {
    match retry {
        Some(policy) => retry::retry_operation(
            || setup_multiplexer_attempt(bearer, address),
            &retry::Policy {
                max_retries: policy.connection_max_retries,
                backoff_unit: Duration::from_secs(1),
                backoff_factor: 2,
                max_backoff: Duration::from_secs(policy.connection_max_backoff as u64),
            },
        ),
        None => setup_multiplexer_attempt(bearer, address),
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum IntersectArg {
    Tip,
    Origin,
    Point(PointArg),
    Fallbacks(Vec<PointArg>),
}

#[derive(Deserialize, Debug, Clone)]
pub struct FinalizeConfig {
    max_block_quantity: Option<u64>,
    max_block_slot: Option<u64>,
    until_hash: Option<String>,
}

pub fn should_finalize(
    config: &Option<FinalizeConfig>,
    last_point: &Point,
    block_count: u64,
) -> bool {
    let config = match config {
        Some(x) => x,
        None => return false,
    };

    if let Some(max) = config.max_block_quantity {
        if block_count >= max {
            return true;
        }
    }

    if let Some(max) = config.max_block_slot {
        if last_point.slot_or_default() >= max {
            return true;
        }
    }

    if let Some(expected) = &config.until_hash {
        if let Point::Specific(_, current) = last_point {
            return expected == &hex::encode(current);
        }
    }

    false
}

pub(crate) fn intersect_starting_point<O>(
    client: &mut chainsync::Client<StdChannel, O>,
    intersect_arg: &Option<IntersectArg>,
    since_arg: &Option<PointArg>,
    utils: &Utils,
) -> Result<Option<Point>, Error>
where
    chainsync::Message<O>: pallas::codec::Fragment,
{
    let cursor = utils.get_cursor_if_any();

    match cursor {
        Some(cursor) => {
            log::info!("found persisted cursor, will use as starting point");
            let desired = cursor.try_into()?;
            let (point, _) = client.find_intersect(vec![desired])?;

            Ok(point)
        }
        None => match intersect_arg {
            Some(IntersectArg::Fallbacks(x)) => {
                log::info!("found 'fallbacks' intersect argument, will use as starting point");
                let options: Result<Vec<_>, _> = x.iter().map(|x| x.clone().try_into()).collect();

                let (point, _) = client.find_intersect(options?)?;

                Ok(point)
            }
            Some(IntersectArg::Origin) => {
                log::info!("found 'origin' intersect argument, will use as starting point");

                let point = client.intersect_origin()?;

                Ok(Some(point))
            }
            Some(IntersectArg::Point(x)) => {
                log::info!("found 'point' intersect argument, will use as starting point");
                let options = vec![x.clone().try_into()?];

                let (point, _) = client.find_intersect(options)?;

                Ok(point)
            }
            Some(IntersectArg::Tip) => {
                log::info!("found 'tip' intersect argument, will use as starting point");

                let point = client.intersect_tip()?;

                Ok(Some(point))
            }
            None => match since_arg {
                Some(x) => {
                    log::info!("explicit 'since' argument, will use as starting point");
                    log::warn!("`since` value is deprecated, please use `intersect`");
                    let options = vec![x.clone().try_into()?];

                    let (point, _) = client.find_intersect(options)?;

                    Ok(point)
                }
                None => {
                    log::info!("no starting point specified, will use tip of chain");

                    let point = client.intersect_tip()?;

                    Ok(Some(point))
                }
            },
        },
    }
}

pub fn unknown_block_to_events(writer: &EventWriter, body: &Vec<u8>) -> Result<(), Error> {
    match probe::block_era(body) {
        probe::Outcome::Matched(era) => match era {
            Era::Byron => {
                writer
                    .crawl_from_byron_cbor(body)
                    .ok_or_warn("error crawling byron block for events");
            }
            Era::Allegra | Era::Alonzo | Era::Mary | Era::Shelley => {
                writer
                    .crawl_from_shelley_cbor(body, era.into())
                    .ok_or_warn("error crawling alonzo-compatible block for events");
            }
            Era::Babbage => {
                writer
                    .crawl_from_babbage_cbor(body)
                    .ok_or_warn("error crawling babbage block for events");
            }
            x => {
                return Err(format!("This version of Oura can't handle era: {x}").into());
            }
        },
        probe::Outcome::EpochBoundary => {
            writer
                .crawl_from_ebb_cbor(body)
                .ok_or_warn("error crawling block for events");
        }
        probe::Outcome::Inconclusive => {
            log::error!("can't infer primitive block from cbor, inconclusive probing. CBOR hex for debugging: {}", hex::encode(body));
        }
    }

    Ok(())
}
