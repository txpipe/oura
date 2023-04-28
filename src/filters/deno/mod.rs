//! A mapper with custom logic from using the Deno runtime

use deno_core::{op, Extension, ModuleSpecifier, OpState};
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker as DenoWorker, WorkerOptions};
use deno_runtime::BootstrapOptions;
use gasket::framework::*;
use gasket::messaging::*;
use gasket::runtime::Tether;
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::trace;

use crate::framework::*;

//pub struct WrappedRuntime(DenoWorker);
//unsafe impl Send for WrappedRuntime {}

pub type WrappedRuntime = DenoWorker;

#[op]
fn op_pop_record(state: &mut OpState) -> Result<serde_json::Value, deno_core::error::AnyError> {
    let r: Record = state.take();
    let j = serde_json::Value::from(r);

    Ok(j)
}

#[op]
fn op_put_record(
    state: &mut OpState,
    value: serde_json::Value,
) -> Result<(), deno_core::error::AnyError> {
    match value {
        serde_json::Value::Null => (),
        _ => state.put(value),
    };

    Ok(())
}

async fn setup_deno(main_module: &PathBuf) -> DenoWorker {
    let ext = Extension::builder("oura")
        .ops(vec![op_pop_record::decl(), op_put_record::decl()])
        .build();

    let empty_module = deno_core::ModuleSpecifier::parse("data:text/javascript;base64,").unwrap();

    let mut deno = DenoWorker::bootstrap_from_options(
        empty_module,
        PermissionsContainer::allow_all(),
        WorkerOptions {
            extensions: vec![ext],
            bootstrap: BootstrapOptions {
                ..Default::default()
            },
            ..Default::default()
        },
    );

    let code = std::fs::read_to_string(main_module).unwrap();

    deno.js_runtime
        .load_side_module(&ModuleSpecifier::parse("oura:mapper").unwrap(), Some(code))
        .await
        .unwrap();

    let res = deno.execute_script("[oura:runtime.js]", include_str!("./runtime.js"));
    deno.run_event_loop(false).await.unwrap();
    res.unwrap();

    deno
}

struct Worker {
    runtime: WrappedRuntime,
}

const SYNC_CALL_SNIPPET: &'static str = r#"Deno[Deno.internal].core.ops.op_put_record(mapEvent(Deno[Deno.internal].core.ops.op_pop_record()));"#;
const ASYNC_CALL_SNIPPET: &'static str = r#"mapEvent(Deno[Deno.internal].core.ops.op_pop_record()).then(x => Deno[Deno.internal].core.ops.op_put_record(x));"#;

impl Worker {
    async fn map_record(
        &mut self,
        script: &str,
        record: Record,
    ) -> Result<Option<serde_json::Value>, String> {
        let deno = &mut self.runtime;

        deno.js_runtime.op_state().borrow_mut().put(record);

        let res = deno.execute_script("<anon>", script);

        deno.run_event_loop(false).await.unwrap();

        res.unwrap();

        let out = deno.js_runtime.op_state().borrow_mut().try_take();
        trace!(?out, "deno mapping finished");

        Ok(out)
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker for Worker {
    type Unit = ChainEvent;
    type Stage = Stage;

    async fn bootstrap(stage: &Self::Stage) -> Result<Self, WorkerError> {
        let runtime = setup_deno(&stage.main_module).await;

        Ok(Self { runtime })
    }

    async fn schedule(
        &mut self,
        stage: &mut Self::Stage,
    ) -> Result<WorkSchedule<Self::Unit>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;

        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(
        &mut self,
        unit: &Self::Unit,
        stage: &mut Self::Stage,
    ) -> Result<(), WorkerError> {
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

pub struct Stage {
    ops_count: gasket::metrics::Counter,
    main_module: PathBuf,
    call_snippet: &'static str,
    input: MapperInputPort,
    output: MapperOutputPort,
}

impl gasket::framework::Stage for Stage {
    fn name(&self) -> &str {
        "mapper"
    }

    fn policy(&self) -> gasket::runtime::Policy {
        gasket::runtime::Policy::default()
    }

    fn register_metrics(&self, registry: &mut gasket::metrics::Registry) {
        registry.track_counter("ops_count", &self.ops_count);
    }
}

impl Stage {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter);
    }

    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        self.output.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage::<Worker>(self);

        Ok(vec![worker_tether])
    }

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
