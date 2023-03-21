//! A mapper with custom logic from using the Deno runtime

use deno_core::{op, Extension, ModuleSpecifier, OpState};
use deno_runtime::permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker, WorkerOptions};
use gasket::{messaging::*, runtime::Tether};
use serde::Deserialize;
use serde_json::json;
use std::ops::Deref;
use std::ops::DerefMut;
use tracing::debug;

use crate::framework::*;

//struct WrappedRuntime(JsRuntime);
struct WrappedRuntime(MainWorker);

unsafe impl Send for WrappedRuntime {}

impl Deref for WrappedRuntime {
    type Target = MainWorker;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WrappedRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[op]
fn op_pop_record(state: &mut OpState) -> Result<serde_json::Value, deno_core::error::AnyError> {
    let r: Record = state.take();

    let j = match r {
        Record::CborBlock(x) => json!({ "len": x.len() as u32 }),
        _ => todo!(),
    };

    Ok(j)
}

#[op]
fn op_put_record(
    state: &mut OpState,
    value: serde_json::Value,
) -> Result<(), deno_core::error::AnyError> {
    state.put(value);

    Ok(())
}

struct Worker {
    ops_count: gasket::metrics::Counter,
    runtime: Option<WrappedRuntime>,
    main_module: ModuleSpecifier,
    input: MapperInputPort,
    output: MapperOutputPort,
}

impl Worker {
    fn eval_apply(&mut self, record: Record) -> Result<Option<serde_json::Value>, String> {
        let deno = self.runtime.as_mut().unwrap();

        {
            deno.js_runtime.op_state().borrow_mut().put(record);
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let res = deno.execute_script(
                "<anon>",
                r#"
Deno[Deno.internal].core.ops.op_put_record(mapEvent(Deno[Deno.internal].core.ops.op_pop_record()));
"#,
            );

            deno.run_event_loop(false).await.unwrap();

            res.unwrap();
        });

        let out = deno.js_runtime.op_state().borrow_mut().try_take();
        debug!(?out, "deno mapping finished");

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

        let mut worker = MainWorker::bootstrap_from_options(
            self.main_module.clone(),
            PermissionsContainer::allow_all(),
            WorkerOptions {
                extensions: vec![ext],
                ..Default::default()
            },
        );

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        rt.block_on(async {
            worker.execute_main_module(&self.main_module).await.unwrap();
        });

        self.runtime = Some(WrappedRuntime(worker));

        Ok(())
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;

        match msg.payload {
            ChainEvent::Apply(p, r) => {
                let mapped = self.eval_apply(r).unwrap();

                if let Some(mapped) = mapped {
                    self.ops_count.inc(1);
                    self.output
                        .send(ChainEvent::Apply(p, Record::GenericJson(mapped)).into())?;
                }
            }
            ChainEvent::Undo(p, r) => todo!(),
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
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        let main_module =
            deno_core::resolve_path(&self.main_module, &ctx.current_dir).map_err(Error::config)?;

        let worker = Worker {
            main_module,
            ops_count: Default::default(),
            runtime: Default::default(),
            input: Default::default(),
            output: Default::default(),
        };

        Ok(Bootstrapper(worker))
    }
}
