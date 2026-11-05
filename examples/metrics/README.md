# Metrics endpoint

Expose pipeline metrics over a Prometheus-compatible HTTP endpoint by enabling the
`[metrics]` block. The pipeline itself does no useful work — it uses the `Noop` sink so the
focus stays on observability.

## Pipeline

```mermaid
flowchart LR
  src[N2N source] --> sink[Noop sink]
  src -. metrics .-> m([Prometheus endpoint])
  sink -. metrics .-> m
```

- **Source** — `N2N`: mainnet relay, starting from the `Point` in `[intersect]`.
- **Sink** — `Noop`: discards events.
- **Metrics** — the `[metrics]` block starts a Prometheus endpoint reporting throughput,
  chain position, and other counters.

## Run

```sh
cd examples/metrics
oura daemon --config daemon.toml
```

Then scrape the metrics endpoint (default `http://localhost:9186/metrics`).
