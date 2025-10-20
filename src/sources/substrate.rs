use std::time::Duration;
use tokio::time::sleep;
use gasket::framework::*;
use serde::Deserialize;
use tracing::{debug, info};

use subxt::{backend::StreamOf, blocks::Block, OnlineClient, SubstrateConfig};

use crate::framework::*;

type SubxtBlock = Block<SubstrateConfig, OnlineClient<SubstrateConfig>>;

pub struct Worker {
    #[allow(dead_code)]
    api: OnlineClient<SubstrateConfig>,
    subscription: StreamOf<Result<SubxtBlock, subxt::Error>>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");

        let api = OnlineClient::<SubstrateConfig>::from_url(&stage.config.url).await.map_err(|e| {
            debug!("Failed to connect to Substrate node: {}", e);
            WorkerError::Panic
        })?;

        let subscription = api.blocks().subscribe_best().await.map_err(|e| {
            debug!("Failed to subscribe to new blocks: {}", e);
            WorkerError::Panic
        })?;

        Ok(Self { api, subscription })
    }

    async fn schedule(&mut self, _: &mut Stage) -> Result<WorkSchedule<SubxtBlock>, WorkerError> {
        info!("awaiting next block (blocking)");

        if let Some(Ok(block)) = self.subscription.next().await {
            debug!("new block received: {:?}", block.hash());
            return Ok(WorkSchedule::Unit(block));
        }

        sleep(Duration::from_secs(5)).await;

        Ok(WorkSchedule::Idle)
    }

    async fn execute(&mut self, block: &SubxtBlock, stage: &mut Stage) -> Result<(), WorkerError> {
        debug!("processing substrate block");
        
        let block_number = block.number().into();
        let block_hash = block.hash();
        
        info!("new block #{}: {:?}", block_number, block_hash);

        // Extract header information
        let header = block.header();
        let parent_hash = header.parent_hash;
        let state_root = header.state_root;
        let extrinsics_root = header.extrinsics_root;

        // Parse extrinsics with detailed information
        let extrinsics_result = block.extrinsics().await.or_retry()?;
        let mut parsed_extrinsics = Vec::new();
        
        for (index, extrinsic) in extrinsics_result.iter().enumerate() {
            let ext_hash = format!("{:?}", extrinsic.hash());
            let ext_bytes = extrinsic.bytes();
            
            // Extract signature info (if signed)
            let signature_info = match (extrinsic.address_bytes(), extrinsic.signature_bytes()) {
                (Some(addr), Some(sig)) => {
                    let address = hex::encode(addr);
                    let signature = hex::encode(sig);
                    
                    Some(substrate::SignatureInfo {
                        address,
                        signature,
                        extra: None, // Could be enhanced to parse signed extensions
                    })
                }
                _ => None,
            };
            
            parsed_extrinsics.push(substrate::Extrinsic {
                index: index as u32,
                hash: ext_hash,
                bytes: Some(hex::encode(ext_bytes)),
                signature: signature_info,
            });
        }

        let extrinsics_count = parsed_extrinsics.len();

        // Create the parsed block structure
        let parsed_block = substrate::ParsedBlock {
            header: substrate::BlockHeader {
                number: block_number,
                hash: format!("{:?}", block_hash),
                parent_hash: format!("{:?}", parent_hash),
                state_root: format!("{:?}", state_root),
                extrinsics_root: format!("{:?}", extrinsics_root),
            },
            extrinsics: parsed_extrinsics,
            extrinsics_count,
        };

        info!("block #{} parsed with {} extrinsics", block_number, extrinsics_count);

        // Create the point for this block
        let point = pallas::network::miniprotocols::Point::Specific(
            block_number,
            block_hash.0.to_vec(),
        );

        // Create and send the chain event
        let event = ChainEvent::Apply(
            point,
            Record::Substrate(substrate::Record::ParsedBlock(Box::new(parsed_block))),
        );
        
        stage.output.send(event.into()).await.or_panic()?;
        stage.ops_count.inc(1);
        stage.chain_tip.set(block_number as i64);

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "source", unit = "SubxtBlock", worker = "Worker")]
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
