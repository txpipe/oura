# Oura

Oura is a Rust pipeline that follows a Cardano chain and sends events to pluggable
destinations. The `main` branch is v2; v1 is maintained on `lts/v1`.

## Layout

| path | what's there |
|---|---|
| `src/sources/` | where events come from: `n2n`, `n2c`, `u5c`, `hydra`, `s3`, `mithril` |
| `src/filters/` | stages that transform events: `split_block`, `parse_cbor`, `select`, `legacy_v1`, `into_json`, `rollback_buffer`, `work_stats`, `wasm_plugin`, `noop` |
| `src/sinks/` | where events go, one module per destination |
| `src/framework/` | `ChainEvent`, `Record`, `Context` and the port types shared by every stage |
| `src/daemon/` | reads `daemon.toml` and wires the stages together |
| `src/bin/oura/` | the CLI: `daemon`, `watch`, `dump` |
| `docs/v2/` | user documentation, published at https://docs.txpipe.io/oura/v2 |
| `examples/` | runnable pipelines, each with a `daemon.toml` |
| `tests/` | integration tests, including the protocol compatibility fixtures |

## Commands

`protoc` must be installed. These match CI:

```sh
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features --locked
```

Use `cargo test --test protocol_compat` for only the protocol checks.

## Working on a stage

- Each stage is a module with a `Config`, a `Stage` and a `Worker`, built on
  [Gasket](https://github.com/construkts/gasket-rs). New stages are registered in the
  `Config` and `Bootstrapper` enums in that folder's `mod.rs`.
- Optional integrations sit behind Cargo features in `Cargo.toml`. Keep new heavy
  dependencies optional.
- A new or changed stage needs a page in `docs/v2/{sources,filters,sinks}/`, and an
  example in `examples/` if it has its own configuration.
- Config keys are user-facing. To rename one, keep the old name with `#[serde(alias)]`.

## Protocol compatibility

- Block and transaction decoding comes from [Pallas](https://github.com/txpipe/pallas).
  Support for a new era starts there.
- `tests/protocol_compat.rs` runs real blocks through the filter stages. When a new
  protocol version goes live, add a block to `tests/protocol/`, list it in that folder's
  README and add it to the test.
- Keep [the compatibility page](docs/v2/reference/protocol_compatibility.mdx) in step.

## Pull requests

- Use conventional commit titles. The changelog is generated from them with `git-cliff`,
  so don't edit `CHANGELOG.md` by hand.
- Keep each pull request to one concern.
- All work happens on the public repository.

## Using Oura from another project

Start with [the integration guide](docs/v2/guides/integrating_oura.mdx), or
[`llms.txt`](llms.txt) for an index of the documentation.
