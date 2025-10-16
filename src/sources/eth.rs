use alloy::eips::BlockId;
use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
};
use alloy::providers::{Identity, Provider, ProviderBuilder, RootProvider, WsConnect};
use alloy::pubsub::SubscriptionStream;
use alloy::rpc::types::Header;
use futures_util::StreamExt;
use gasket::framework::*;
use serde::Deserialize;
use tracing::{debug, info};

use crate::framework::*;

pub struct Worker {
    stream: SubscriptionStream<Header>,
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

        let stream = subscription.into_stream();

        Ok(Self { stream, provider })
    }

    async fn schedule(&mut self, _: &mut Stage) -> Result<WorkSchedule<Header>, WorkerError> {
        info!("awaiting next block (blocking)");
        if let Some(header) = self.stream.next().await {
            return Ok(WorkSchedule::Unit(header));
        }

        Ok(WorkSchedule::Idle)
    }

    async fn execute(&mut self, header: &Header, stage: &mut Stage) -> Result<(), WorkerError> {
        debug!(hash = header.hash.to_string(), "chain sync roll forward");

        let block_id = BlockId::hash(header.hash);
        info!("requesting next block");
        if let Some(block) = self.provider.get_block(block_id).await.or_retry()? {
            let event = ChainEvent::Apply(
                pallas::network::miniprotocols::Point::Specific(
                    block.header.number,
                    block.header.hash.to_vec(),
                ),
                Record::Ethereum(ethereum::Record::ParsedBlock(Box::new(block))),
            );
            stage.output.send(event.into()).await.or_panic()?;
        }

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "source", unit = "Header", worker = "Worker")]
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
