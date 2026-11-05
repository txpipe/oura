# Elasticsearch sink

Decode transactions and index them into Elasticsearch.

## Pipeline

```mermaid
flowchart LR
  src[N2N source] --> f1[SplitBlock] --> f2[ParseCbor] --> sink[ElasticSearch sink]
```

- **Source** — `N2N`: mainnet relay, starting from the chain tip.
- **Filters**
  - `SplitBlock`: breaks each block into individual transactions.
  - `ParseCbor`: decodes the raw transaction CBOR into structured records.
- **Sink** — `ElasticSearch`: indexes documents into `index` on the cluster at `url`
  (`idempotency = true` makes writes safe to replay).

## Prerequisites

- Built with the `elasticsearch` feature.
- A running Elasticsearch cluster — a `docker-compose.yaml` is included.

```sh
docker compose up -d
```

## Run

```sh
cd examples/elasticsearch
cargo run --features elasticsearch --bin oura -- daemon --config daemon.toml
```

(or `oura daemon --config daemon.toml` with a binary built with the `elasticsearch` feature.)
