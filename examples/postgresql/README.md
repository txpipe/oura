# PostgreSQL sink

Split blocks into transactions and persist their raw CBOR into a PostgreSQL database using
templated SQL statements.

## Pipeline

```mermaid
flowchart LR
  src[N2N source] --> f1[SplitBlock] --> sink[SqlDb sink]
```

- **Source** — `N2N`: mainnet relay, starting from the `Point` in `[intersect]`.
- **Filters** — `SplitBlock`: breaks each block into individual transactions.
- **Sink** — `SqlDb`: runs the `apply_template` / `undo_template` / `reset_template`
  statements against the `postgres://` connection, storing each transaction's slot and CBOR.
  Rollbacks and resets are handled by the undo/reset templates.

## Companion files

- `docker-compose.yml` — a local PostgreSQL instance.
- `init.sql` — creates the `txs` table (and slot index) the templates write to.

## Prerequisites

- Built with the `sql` feature.

## Setup

1. Start PostgreSQL:

   ```sh
   docker compose up -d
   ```

2. Create the table:

   ```sh
   psql -h localhost -p 5432 -U postgres -f init.sql
   ```

## Run

```sh
cd examples/postgresql
cargo run --features sql --bin oura -- daemon --config daemon.toml
```

(or `oura daemon --config daemon.toml` with a binary built with the `sql` feature.)
