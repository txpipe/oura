//! A mapper with custom logic from using the Deno runtime

use deno_core::{op, Extension, ModuleSpecifier, OpState};
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker as DenoWorker, WorkerOptions};
use deno_runtime::BootstrapOptions;
use gasket::{messaging::*, runtime::Tether};
use pallas::network::miniprotocols::Point;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::trace;

use crate::framework::*;

pub struct WrappedRuntime(DenoWorker);

unsafe impl Send for WrappedRuntime {}

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
    ops_count: gasket::metrics::Counter,
    runtime: Option<WrappedRuntime>,
    main_module: PathBuf,
    call_snippet: &'static str,
    input: MapperInputPort,
    output: MapperOutputPort,
}

const SYNC_CALL_SNIPPET: &'static str = r#"Deno[Deno.internal].core.ops.op_put_record(mapEvent(Deno[Deno.internal].core.ops.op_pop_record()));"#;
const ASYNC_CALL_SNIPPET: &'static str = r#"mapEvent(Deno[Deno.internal].core.ops.op_pop_record()).then(x => Deno[Deno.internal].core.ops.op_put_record(x));"#;

impl Worker {
    async fn map_record(&mut self, record: Record) -> Result<Option<serde_json::Value>, String> {
        let deno = &mut self.runtime.as_mut().unwrap().0;

        deno.js_runtime.op_state().borrow_mut().put(record);

        let res = deno.execute_script("<anon>", self.call_snippet);

        deno.run_event_loop(false).await.unwrap();

        res.unwrap();

        let out = deno.js_runtime.op_state().borrow_mut().try_take();
        trace!(?out, "deno mapping finished");

        Ok(out)
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

#[async_trait::async_trait(?Send)]
impl gasket::runtime::Worker for Worker {
    type WorkUnit = ChainEvent;

    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    async fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let deno = setup_deno(&self.main_module).await;
        self.runtime = Some(WrappedRuntime(deno));

        Ok(())
    }

    async fn schedule(&mut self) -> gasket::runtime::ScheduleResult<Self::WorkUnit> {
        let msg = self.input.recv().await?;

        Ok(gasket::runtime::WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &Self::WorkUnit) -> Result<(), gasket::error::Error> {
        match unit {
            ChainEvent::Apply(p, r) => {
                let mapped = self.map_record(r.clone()).await.unwrap();
                self.fanout(p.clone(), mapped).await?
            }
            ChainEvent::Undo(..) => todo!(),
            ChainEvent::Reset(p) => {
                self.output.send(ChainEvent::reset(p.clone())).await?;
            }
        };

        Ok(())
    }
}

pub struct Bootstrapper(Worker);

impl Bootstrapper {
    pub fn connect_input(&mut self, adapter: InputAdapter) {
        self.0.input.connect(adapter);
    }

    pub fn connect_output(&mut self, adapter: OutputAdapter) {
        self.0.output.connect(adapter);
    }

    pub fn spawn(self) -> Result<Vec<Tether>, Error> {
        let worker_tether = gasket::runtime::spawn_stage(
            self.0,
            gasket::runtime::Policy::default(),
            Some("mapper"),
        );

        Ok(vec![worker_tether])
    }
}

#[derive(Deserialize)]
pub struct Config {
    main_module: String,
    use_async: bool,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Bootstrapper, Error> {
        // let main_module =
        //    deno_core::resolve_path(&self.main_module,
        // &ctx.current_dir).map_err(Error::config)?;

        let worker = Worker {
            //main_module,
            main_module: PathBuf::from(self.main_module),
            call_snippet: if self.use_async {
                ASYNC_CALL_SNIPPET
            } else {
                SYNC_CALL_SNIPPET
            },
            runtime: None,
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
        };

        Ok(Bootstrapper(worker))
    }
}
