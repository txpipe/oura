# WASM Basic

This example shows how to build a custom Oura plugin using Golang.

The code under `./extract_fee` contains the Golang source code for the plugin. The scope of this very basic plugin is to extract the fee value of each transaction.

## Requirements

- [tinygo](https://tinygo.org/getting-started/install/)

> While the core Go toolchain has support to target WebAssembly, we find tinygo to work well for plug-in code.

## Procedure

1. Build the plugin

Run the following command from inside the `./extract_fee` directory to compile the Golang source into a WASM module using tinigo.

```sh
tinygo build -o plugin.wasm -target wasi main.go
```

The Golang code relies on a plugin system called [extism](https://github.com/extism) that provides several extra features which are not reflected in this example. To read more about how to use Extism in go, refer to the [official docs](https://github.com/extism/go-pdk).

1. Run Oura using the plugin

Run Oura using the `daemon.toml` config in this example that already points to the compiled WASM module generated in the previous step.

```sh
cargo run --features wasm --bin oura -- daemon --config ./daemon.toml
```

> Note that wasm plugins require the Cargo feature flag named `wasm`

You should notice that the events piped in stdout show numbers that represent the fees of each transaction processed.