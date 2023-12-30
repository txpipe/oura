//! An utility to keep track of the progress of the pipeline as a whole

use anyhow::{Context, Result};
use gasket::{metrics::Reading, runtime::Tether};
use prometheus_exporter_base::{
    prelude::{Authorization, ServerOptions},
    render_prometheus, MetricType, PrometheusInstance, PrometheusMetric,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufWriter, Write},
    net::SocketAddr,
    sync::Arc,
};
use tracing::warn;

use crate::daemon::Runtime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub address: Option<String>,
}

pub fn render_tether(tether: &Tether, render: &mut impl Write) -> Result<()> {
    let readings = tether.read_metrics()?;

    let stage = tether.name();

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();

    for (name, value) in readings {
        let full_name = format!("{stage}_{name}");

        let metric_type = match value {
            Reading::Count(_) => MetricType::Counter,
            Reading::Gauge(_) => MetricType::Gauge,
            Reading::Message(_) => MetricType::Summary,
        };

        let mut pc = PrometheusMetric::build()
            .with_name(&full_name)
            .with_help("some specific help")
            .with_metric_type(metric_type)
            .build();

        match value {
            Reading::Count(x) => {
                pc.render_and_append_instance(
                    &PrometheusInstance::new().with_value(x).with_timestamp(ts),
                );
            }
            Reading::Gauge(x) => {
                pc.render_and_append_instance(
                    &PrometheusInstance::new().with_value(x).with_timestamp(ts),
                );
            }
            Reading::Message(msg) => {
                warn!(msg, "can't render message metrics to prometheous");
            }
        };

        writeln!(render, "{}", pc.render())?;
    }

    Ok(())
}

fn render(runtime: &Runtime) -> Result<String> {
    warn!("rendering");
    let mut buf = BufWriter::new(Vec::new());

    runtime
        .all_tethers()
        .try_for_each(|x| render_tether(x, &mut buf))?;

    let bytes = buf.into_inner()?;
    let string = String::from_utf8(bytes)?;

    Ok(string)
}

pub async fn initialize(config: Config, runtime: Arc<Runtime>) -> Result<()> {
    let addr: SocketAddr = config
        .address
        .as_deref()
        .unwrap_or("0.0.0.0:9186")
        .parse()
        .context("parsing binding config")?;

    let server_options = ServerOptions {
        addr,
        authorization: Authorization::None,
    };

    render_prometheus(server_options, runtime, |_, options| async move {
        Ok(render(&options).unwrap())
    })
    .await;

    Ok(())
}
