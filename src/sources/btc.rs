use std::time::Duration;

use gasket::framework::*;
use serde::{Deserialize};
use serde_json::Value;
use tokio::time::sleep;
use tracing::debug;
use bitcoind_async_client::{traits::Reader, Client};
use corepc_types::bitcoin::{Block, BlockHash};

use crate::framework::*;


pub struct Worker {
    rpc_client: Client,
    rpc_interval: u64,
    last_block_hash: BlockHash,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("Creating client with: {}", &stage.config.rpc_url);

        // Some RPC clients require authentication, so we provide default values if none are set.
        // Library doesn't support "no-auth" mode, so we provide dummy values which work for public nodes.
        let user = stage.config.rpc_user.clone().unwrap_or("no-auth".to_string());
        let password = stage.config.rpc_password.clone().unwrap_or("password".to_string());
        let rpc_client = Client::new(stage.config.rpc_url.clone(), user, password, None, None)
            .map_err(|e| {
                debug!("Failed to create RPC client: {}", e);

                WorkerError::Panic
            })?;

        let blockchain_info = rpc_client.get_blockchain_info().await.map_err(|e| {
            debug!("Failed to connect to RPC: {}", e);
            WorkerError::Panic
        })?;

        Ok(Self {
            rpc_client,
            rpc_interval: stage.config.rpc_interval.unwrap_or(30), // Default 30 seconds
            last_block_hash: blockchain_info.best_block_hash
        })
    }

    async fn schedule(&mut self, _: &mut Stage) -> Result<WorkSchedule<Value>, WorkerError> {
        debug!("Scheduling next work unit...");

        loop {
            let blockchain_info = self.rpc_client.get_blockchain_info().await.map_err(|e| {
                debug!("Failed to fetch blockchain info from RPC: {}", e);
                WorkerError::Panic
            })?;

            if blockchain_info.best_block_hash.eq(&self.last_block_hash) {
                debug!("No new blocks found");
                // Wait for the next polling interval
                sleep(Duration::from_secs(self.rpc_interval)).await;
            } else {
                debug!("New block found: {}", blockchain_info.best_block_hash);
                self.last_block_hash = blockchain_info.best_block_hash;
                break;
            }
        }

        // Get Block info
        let block = self.rpc_client.get_block(&self.last_block_hash).await.map_err(|e| {
            debug!("Failed to fetch block from RPC: {}", e);
            WorkerError::Panic
        })?;

        Ok(WorkSchedule::Unit(serde_json::to_value(&block).unwrap_or(Value::Null)))
    }
    

    async fn execute(&mut self, json_data: &Value, stage: &mut Stage) -> Result<(), WorkerError> {
        debug!("Processing block info...");

        if let Ok(block) = serde_json::from_value::<Block>(json_data.clone()) {
            let block_hash = block.block_hash();
            let height = block.bip34_block_height().map_err(|e| {
                debug!("Failed to get block height: {}", e);

                WorkerError::Panic
            })?;

            debug!(hash = block_hash.to_string(), height = height,  "new block received");

            let event = ChainEvent::Apply(
                pallas::network::miniprotocols::Point::Specific(
                    height,
                    hex::decode(block_hash.to_string()).unwrap_or_default()
                ),
                Record::Bitcoin(bitcoin::Record::ParsedBlock(Box::new(block))),
            );
            
            stage.output.send(event.into()).await.or_panic()?;
            stage.ops_count.inc(1);
            stage.chain_tip.set(height as i64);
        }

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "source-btc-rpc", unit = "Value", worker = "Worker")]
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
    /// JSON-RPC client for the Bitcoin Core daemon
    pub rpc_url: String,
    pub rpc_user: Option<String>,
    pub rpc_password: Option<String>,
    /// Polling interval in seconds
    pub rpc_interval: Option<u64>,
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