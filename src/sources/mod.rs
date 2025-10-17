use gasket::runtime::Tether;
use serde::Deserialize;

use crate::framework::*;

//#[cfg(target_family = "unix")]
//pub mod n2c;

pub mod n2c;
pub mod n2n;

#[cfg(feature = "eth")]
pub mod eth;

#[cfg(feature = "btc")]
pub mod btc;

#[cfg(feature = "substrate")]
pub mod substrate;

#[cfg(feature = "hydra")]
pub mod hydra;

#[cfg(feature = "u5c")]
pub mod u5c;

#[cfg(feature = "aws")]
pub mod s3;

#[cfg(feature = "mithril")]
pub mod mithril;

pub enum Bootstrapper {
    N2N(n2n::Stage),

    #[cfg(target_family = "unix")]
    N2C(n2c::Stage),

    #[cfg(feature = "eth")]
    EthereumRpc(eth::Stage),

    #[cfg(feature = "btc")]
    BitcoinRpc(btc::Stage),

    #[cfg(feature = "substrate")]
    SubstrateRpc(substrate::Stage),

    #[cfg(feature = "hydra")]
    Hydra(hydra::Stage),

    #[cfg(feature = "u5c")]
    U5C(u5c::Stage),

    #[cfg(feature = "aws")]
    S3(s3::Stage),

    #[cfg(feature = "mithril")]
    Mithril(mithril::Stage),
}

impl Bootstrapper {
    pub fn borrow_output(&mut self) -> &mut SourceOutputPort {
        match self {
            Bootstrapper::N2N(p) => &mut p.output,

            #[cfg(target_family = "unix")]
            Bootstrapper::N2C(p) => &mut p.output,

            #[cfg(feature = "eth")]
            Bootstrapper::EthereumRpc(p) => &mut p.output,

            #[cfg(feature = "btc")]
            Bootstrapper::BitcoinRpc(p) => &mut p.output,

            #[cfg(feature = "substrate")]
            Bootstrapper::SubstrateRpc(p) => &mut p.output,

            #[cfg(feature = "hydra")]
            Bootstrapper::Hydra(p) => &mut p.output,

            #[cfg(feature = "u5c")]
            Bootstrapper::U5C(p) => &mut p.output,

            #[cfg(feature = "aws")]
            Bootstrapper::S3(p) => &mut p.output,

            #[cfg(feature = "mithril")]
            Bootstrapper::Mithril(p) => &mut p.output,
        }
    }

    pub fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::N2N(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(target_family = "unix")]
            Bootstrapper::N2C(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "eth")]
            Bootstrapper::EthereumRpc(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "btc")]
            Bootstrapper::BitcoinRpc(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "substrate")]
            Bootstrapper::SubstrateRpc(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "hydra")]
            Bootstrapper::Hydra(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "u5c")]
            Bootstrapper::U5C(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "aws")]
            Bootstrapper::S3(x) => gasket::runtime::spawn_stage(x, policy),

            #[cfg(feature = "mithril")]
            Bootstrapper::Mithril(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    N2N(n2n::Config),

    #[cfg(target_family = "unix")]
    N2C(n2c::Config),

    #[cfg(feature = "eth")]
    #[serde(rename = "ethereum-rpc")]
    EthereumRpc(eth::Config),

    #[cfg(feature = "btc")]
    #[serde(rename = "bitcoin-rpc")]
    BitcoinRpc(btc::Config),

    #[cfg(feature = "substrate")]
    #[serde(rename = "substrate-rpc")]
    SubstrateRpc(substrate::Config),

    #[cfg(feature = "hydra")]
    Hydra(hydra::Config),

    #[cfg(feature = "u5c")]
    U5C(u5c::Config),

    #[cfg(feature = "aws")]
    S3(s3::Config),

    #[cfg(feature = "mithril")]
    Mithril(mithril::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::N2N(c) => Ok(Bootstrapper::N2N(c.bootstrapper(ctx)?)),

            #[cfg(target_family = "unix")]
            Config::N2C(c) => Ok(Bootstrapper::N2C(c.bootstrapper(ctx)?)),

            #[cfg(feature = "eth")]
            Config::EthereumRpc(c) => Ok(Bootstrapper::EthereumRpc(c.bootstrapper(ctx)?)),

            #[cfg(feature = "btc")]
            Config::BitcoinRpc(c) => Ok(Bootstrapper::BitcoinRpc(c.bootstrapper(ctx)?)),

            #[cfg(feature = "substrate")]
            Config::SubstrateRpc(c) => Ok(Bootstrapper::SubstrateRpc(c.bootstrapper(ctx)?)),

            #[cfg(feature = "hydra")]
            Config::Hydra(c) => Ok(Bootstrapper::Hydra(c.bootstrapper(ctx)?)),

            #[cfg(feature = "u5c")]
            Config::U5C(c) => Ok(Bootstrapper::U5C(c.bootstrapper(ctx)?)),

            #[cfg(feature = "aws")]
            Config::S3(c) => Ok(Bootstrapper::S3(c.bootstrapper(ctx)?)),

            #[cfg(feature = "mithril")]
            Config::Mithril(c) => Ok(Bootstrapper::Mithril(c.bootstrapper(ctx)?)),
        }
    }
}
