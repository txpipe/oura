# Hydra source

Stream transactions from a [Hydra](https://hydra.family) head over its WebSocket API and
print them to standard output.

## Pipeline

```mermaid
flowchart LR
  src[Hydra source] --> sink[Stdout sink]
```

- **Source** — `Hydra`: connects to the head's `ws_url` (network `magic`), starting from
  `Origin`.
- **Sink** — `Stdout`: prints each event.

## Prerequisites

- A running Hydra node exposing its WebSocket API (default `ws://127.0.0.1:4001`).
- Built with the `hydra` feature.

## Run

```sh
cd examples/hydra
oura daemon --config daemon.toml
```

From source:

```sh
cargo run --features hydra --bin oura -- daemon --config daemon.toml
```
