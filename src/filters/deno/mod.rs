//! A mapper with custom logic from using the Deno runtime

use deno_core::{op, Extension, ModuleSpecifier, OpState};
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker as DenoWorker, WorkerOptions};
use deno_runtime::BootstrapOptions;
use gasket::{messaging::*, runtime::Tether};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::runtime::Runtime as TokioRuntime;
use tracing::trace;

use crate::framework::*;

struct WrappedRuntime(DenoWorker, TokioRuntime);

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
    fn eval_apply(&mut self, record: Record) -> Result<Option<serde_json::Value>, String> {
        let WrappedRuntime(deno, tokio) = self.runtime.as_mut().unwrap();

        deno.js_runtime.op_state().borrow_mut().put(record);

        tokio.block_on(async {
            let res = deno.execute_script("<anon>", self.call_snippet);

            deno.run_event_loop(false).await.unwrap();

            res.unwrap();
        });

        let out = deno.js_runtime.op_state().borrow_mut().try_take();
        trace!(?out, "deno mapping finished");

        Ok(out)
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let ext = Extension::builder("oura")
            .ops(vec![op_pop_record::decl(), op_put_record::decl()])
            .build();

        let empty_module =
            deno_core::ModuleSpecifier::parse("data:text/javascript;base64,").unwrap();

        let mut worker = DenoWorker::bootstrap_from_options(
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

        let reactor = deno_runtime::tokio_util::create_basic_runtime();

        reactor.block_on(async {
            let code = std::fs::read_to_string(&self.main_module).unwrap();

            worker
                .js_runtime
                .load_side_module(&ModuleSpecifier::parse("oura:mapper").unwrap(), Some(code))
                .await
                .unwrap();

            let res = worker.execute_script("[oura:runtime.js]", include_str!("./runtime.js"));
            worker.run_event_loop(false).await.unwrap();
            res.unwrap();
        });

        self.runtime = Some(WrappedRuntime(worker, reactor));

        Ok(())
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;

        match msg.payload {
            ChainEvent::Apply(p, r) => {
                let mapped = self.eval_apply(r).unwrap();

                if let Some(mapped) = mapped {
                    self.ops_count.inc(1);

                    match mapped {
                        serde_json::Value::Array(items) => {
                            for item in items {
                                self.output.send(
                                    ChainEvent::Apply(p.clone(), Record::GenericJson(item)).into(),
                                )?;
                            }
                        }
                        _ => {
                            self.output
                                .send(ChainEvent::Apply(p, Record::GenericJson(mapped)).into())?;
                        }
                    }
                }
            }
            ChainEvent::Undo(..) => todo!(),
            ChainEvent::Reset(p) => {
                self.output.send(ChainEvent::reset(p))?;
            }
        }

        Ok(gasket::runtime::WorkOutcome::Partial)
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
            ops_count: Default::default(),
            runtime: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(Bootstrapper(worker))
    }
}
