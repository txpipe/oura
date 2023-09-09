//! A mapper with custom logic from using the Deno runtime

use deno_runtime::deno_core::{self, op2, ModuleSpecifier, OpState};
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker as DenoWorker, WorkerOptions};
use gasket::framework::*;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use std::path::PathBuf;

use tracing::trace;

use crate::framework::*;

pub type WrappedRuntime = DenoWorker;

deno_core::extension!(deno_filter, ops = [op_pop_record, op_put_record]);

#[op2]
#[serde]
pub fn op_pop_record(state: &mut OpState) -> Result<serde_json::Value, deno_core::error::AnyError> {
    let r: Record = state.take();
    let j = serde_json::Value::from(r);
    Ok(j)
}

#[op2]
pub fn op_put_record(
    state: &mut OpState,
    #[serde] value: serde_json::Value,
) -> Result<(), deno_core::error::AnyError> {
    match value {
        serde_json::Value::Null => (),
        _ => state.put(value),
    };

    Ok(())
}

async fn setup_deno(main_module: &PathBuf) -> DenoWorker {
    let empty_module = deno_core::ModuleSpecifier::parse("data:text/javascript;base64,").unwrap();

    let mut deno = DenoWorker::bootstrap_from_options(
        empty_module,
        PermissionsContainer::allow_all(),
        WorkerOptions {
            extensions: vec![deno_filter::init_ops()],
            ..Default::default()
        },
    );

    let code = deno_core::FastString::from(std::fs::read_to_string(main_module).unwrap());

    deno.js_runtime
        .load_side_module(&ModuleSpecifier::parse("oura:mapper").unwrap(), Some(code))
        .await
        .unwrap();

    let runtime_code = deno_core::FastString::from_static(include_str!("./runtime.js"));

    let res = deno.execute_script("[oura:runtime.js]", runtime_code);
    deno.run_event_loop(false).await.unwrap();
    res.unwrap();

    deno
}

pub struct Worker {
    runtime: WrappedRuntime,
}

const SYNC_CALL_SNIPPET: &str = r#"Deno[Deno.internal].core.ops.op_put_record(mapEvent(Deno[Deno.internal].core.ops.op_pop_record()));"#;

const ASYNC_CALL_SNIPPET: &str = r#"mapEvent(Deno[Deno.internal].core.ops.op_pop_record()).then(x => Deno[Deno.internal].core.ops.op_put_record(x));"#;

impl Worker {
    async fn map_record(
        &mut self,
        script: &'static str,
        record: Record,
    ) -> Result<Option<serde_json::Value>, String> {
        let deno = &mut self.runtime;

        trace!(?record, "sending record to js runtime");
        deno.js_runtime.op_state().borrow_mut().put(record);

        let script = deno_core::FastString::from_static(script);
        let res = deno.execute_script("<anon>", script);

        deno.run_event_loop(false).await.unwrap();

        res.unwrap();

        let out = deno.js_runtime.op_state().borrow_mut().try_take();
        trace!(?out, "received record from js runtime");

        Ok(out)
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let runtime = setup_deno(&stage.main_module).await;

        Ok(Self { runtime })
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;

        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        match unit {
            ChainEvent::Apply(p, r) => {
                let mapped = self
                    .map_record(stage.call_snippet, r.clone())
                    .await
                    .unwrap();

                stage.fanout(p.clone(), mapped).await.or_panic()?
            }
            ChainEvent::Undo(..) => todo!(),
            ChainEvent::Reset(p) => {
                stage
                    .output
                    .send(ChainEvent::reset(p.clone()))
                    .await
                    .or_panic()?;
            }
        };

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "filter-deno", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    main_module: PathBuf,
    call_snippet: &'static str,

    pub input: MapperInputPort,
    pub output: MapperOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

impl Stage {
    async fn fanout(
        &mut self,
        point: Point,
        mapped: Option<serde_json::Value>,
    ) -> Result<(), gasket::error::Error> {
        if let Some(mapped) = mapped {
            self.ops_count.inc(1);

            match mapped {
                serde_json::Value::Array(items) => {
                    for item in items {
                        self.output
                            .send(
                                ChainEvent::Apply(point.clone(), Record::GenericJson(item)).into(),
                            )
                            .await?;
                    }
                }
                _ => {
                    self.output
                        .send(ChainEvent::Apply(point, Record::GenericJson(mapped)).into())
                        .await?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    main_module: String,
    use_async: bool,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        // let main_module =
        //    deno_core::resolve_path(&self.main_module,
        // &ctx.current_dir).map_err(Error::config)?;

        let stage = Stage {
            //main_module,
            main_module: PathBuf::from(self.main_module),
            call_snippet: if self.use_async {
                ASYNC_CALL_SNIPPET
            } else {
                SYNC_CALL_SNIPPET
            },
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
        };

        Ok(stage)
    }
}
