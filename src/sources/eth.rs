use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
};
use alloy::providers::{Identity, Provider, ProviderBuilder, RootProvider, WsConnect};
use alloy::pubsub::SubscriptionStream;
use alloy::rpc::types::Header;
use futures_util::stream::Take;
use futures_util::StreamExt;
use gasket::framework::*;
use serde::Deserialize;
use tracing::debug;

use crate::framework::*;

pub struct Worker {
    stream: Take<SubscriptionStream<Header>>,
    provider: FillProvider<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        RootProvider,
    >,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");

        let ws = WsConnect::new(&stage.config.url);
        let provider = ProviderBuilder::new().connect_ws(ws).await.or_panic()?;

        let subscription = provider.subscribe_blocks().await.or_panic()?;

        let stream = subscription.into_stream().take(2);

        Ok(Self { stream, provider })
    }

    async fn schedule(&mut self, _: &mut Stage) -> Result<WorkSchedule<Header>, WorkerError> {
        if let Some(header) = self.stream.next().await {
            return Ok(WorkSchedule::Unit(header));
        }

        Ok(WorkSchedule::Idle)
    }

    async fn execute(&mut self, header: &Header, _stage: &mut Stage) -> Result<(), WorkerError> {
        dbg!(header);

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "source-utxorpc", unit = "Header", worker = "Worker")]
pub struct Stage {
    config: Config,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    chain_tip: gasket::metrics::Gauge,

    #[metric]
    current_slot: gasket::metrics::Gauge,
}

#[derive(Deserialize)]
pub struct Config {
    pub url: String,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
            current_slot: Default::default(),
        };

        Ok(stage)
    }
}
