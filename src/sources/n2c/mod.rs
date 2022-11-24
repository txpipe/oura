mod chainsync;
mod transport;

use serde::Deserialize;
use std::time::Duration;

use crate::{bootstrap, crosscut, model, storage};

use gasket::messaging::OutputPort;

#[derive(Deserialize)]
pub struct Config {
    pub path: String,
    pub min_depth: Option<usize>,
}

impl Config {
    pub fn bootstrapper(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
        finalize: &Option<crosscut::FinalizeConfig>,
        policy: &crosscut::policies::RuntimePolicy,
    ) -> Bootstrapper {
        Bootstrapper {
            config: self,
            intersect: intersect.clone(),
            finalize: finalize.clone(),
            policy: policy.clone(),
            chain: chain.clone(),
            output: Default::default(),
        }
    }
}

pub struct Bootstrapper {
    config: Config,
    intersect: crosscut::IntersectConfig,
    finalize: Option<crosscut::FinalizeConfig>,
    policy: crosscut::policies::RuntimePolicy,
    chain: crosscut::ChainWellKnownInfo,
    output: OutputPort<model::RawBlockPayload>,
}

impl Bootstrapper {
    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<model::RawBlockPayload> {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline, cursor: storage::Cursor) {
        pipeline.register_stage(gasket::runtime::spawn_stage(
            self::chainsync::Worker::new(
                self.config.path.clone(),
                self.config.min_depth.unwrap_or(0),
                self.policy,
                self.chain,
                self.intersect,
                self.finalize,
                cursor,
                self.output,
            ),
            gasket::runtime::Policy {
                tick_timeout: Some(Duration::from_secs(600)),
                bootstrap_retry: gasket::retries::Policy {
                    max_retries: 20,
                    backoff_factor: 2,
                    backoff_unit: Duration::from_secs(1),
                    max_backoff: Duration::from_secs(60),
                },
                ..Default::default()
            },
            Some("n2c"),
        ));
    }
}
