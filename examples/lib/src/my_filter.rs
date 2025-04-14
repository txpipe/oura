use gasket::framework::*;
use oura::framework::*;
use tracing::warn;
use utxorpc::spec::cardano::certificate::Certificate;

#[derive(Default, Stage)]
#[stage(name = "my-filter", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: FilterInputPort,
    pub output: FilterOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default)]
pub struct Worker;

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(_: &Stage) -> Result<Self, WorkerError> {
        Ok(Self)
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let record = unit.record().cloned();

        if let Some(record) = record {
            match record {
                Record::ParsedTx(tx) => {
                    if tx.certificates.iter().any(|c| {
                        if let Some(certificate) = &c.certificate {
                            return match certificate {
                                Certificate::VoteDelegCert(cert) => cert.drep.is_some(),
                                _ => false,
                            };
                        }
                        false
                    }) {
                        stage.output.send(unit.clone().into()).await.or_panic()?
                    }
                }
                _ => {
                    warn!("my_filter only supports parsedTx, enable split_block and parse_cbor");
                }
            };
        }

        stage.ops_count.inc(1);

        Ok(())
    }
}
