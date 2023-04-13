<div align="center">
    <img src="assets/logo.svg" alt="Oura" width="500">
    <hr />
        <h2 align="center" style="border-bottom: none">The tail of Cardano</h2>
        <img alt="GitHub" src="https://img.shields.io/github/license/txpipe/oura" />
        <img alt="Crates.io" src="https://img.shields.io/crates/v/oura" />
        <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/txpipe/oura/validate.yml" />
    <hr/>
</div>

> **Warning**
> `main` branch is now tracking **V2**. This new version is a complete overhaul of the processing pipeline, multiple breaking changes. If you're looking for **V1**, you can switch to the long-term support branch named `lts/v1`

## Introduction

We have tools to "explore" the Cardano blockchain, which are useful when you know what you're looking for. We argue that there's a different, complementary use-case which is to "observe" the blockchain and react to particular event patterns.

_Oura_ is a rust-native implementation of a pipeline that connects to the tip of a Cardano node through a combination of _Ouroboros_ mini-protocol (using either a unix socket or tcp bearer), filters the events that match a particular pattern and then submits a succinct, self-contained payload to pluggable observers called "sinks".

Check our [documentation](https://txpipe.github.io/oura) for detailed information on how to start working with _Oura_.

## Etymology

The name of the tool is inspired by the `tail` command available in unix-like systems which is used to display the tail end of a text file or piped data. Cardano's consensus protocol name, _Ouroboros_, is a reference to the ancient symbol depicting a serpent or dragon eating its own tail, which means "tail eating". "Oura" is the ancient greek word for "tail".

## Terminal Output Demo

In this terminal recording we get to see a few mins of live output from a testnet node connected to the terminal sink.

![Terminal Demo](assets/demo.svg)

[watch full asciicast](https://asciinema.org/a/453455)

## Under the Hood

All the heavy lifting required to communicate with the Cardano node is done by the [Pallas](https://github.com/txpipe/pallas) library, which provides an implementation of the Ouroboros multiplexer and a few of the required mini-protocol state-machines (ChainSync and LocalState in particular).

The data pipeline is implemented by the [Gasket](https://github.com/construkts/gasket-rs) library which provides a framework for building staged, event-driven applications. Under this abstraction, each component of the pipeline (aka: _Stage_) runs in its own thread and communicates with other stages by sending messages (very similar to the _Actor pattern_).

## Use Cases

### CLI to Watch Live Transactions

You can run `oura watch <socket>` to print TX data into the terminal from the tip of a local or remote node. It can be useful as a debugging tool for developers or if you're just curious to see what's going on in the network (for example, to see airdrops as they happen or oracles posting new information).

### As a Bridge to Other Persistence Mechanisms

Similar to the well-known db-sync tool provided by IOHK, _Oura_ can be used as a daemon to follow a node and output the data into a different data storage technology more suited for your final use case. The main difference with db-sync is that _Oura_ was designed for easy integration with data-streaming pipelines instead of relational databases.

Given its small memory / cpu footprint, _Oura_ can be deployed side-by-side with your Cardano node even in resource-constrained environments, such as Raspberry PIs.

### As A Trigger Of Custom Actions

_Oura_ running in `daemon` mode can be configured to use custom filters to pinpoint particular transaction patterns and trigger actions whenever it finds a match. For example: send an email when a particular policy / asset combination appears in a transaction; call an AWS Lambda function when a wallet delegates to a particular pool; send a http-call to a webhook each time a metadata key appears in the TX payload;

### As a Library for Custom Scenarios

If the available out-of-the-box features don't satisfy your particular use case, _Oura_ can be used a library in your Rust project to setup tailor-made pipelines. Each component (sources, filters, sinks, etc) in _Oura_ aims at being self-contained and reusable. For example, custom filters and sinks can be built while reusing the existing sources.

## How it Works

Oura is in its essence just a pipeline for processing events. Each stage of the pipeline fulfills a different role:

- Source Stages: are in charge of pulling data from the blockchain and mapping the raw blocks into smaller, more granular events. Each event is then sent through the output port of the stage for further processing.
- Filter Stages: receive individual events from the source stage and apply some sort of transformation to each one. The transformations applied will depend on the particular use case, but they usually revolve around selecting relevant events and enriching them with extra information.
- Sink Stages: receive the final events from the filter stage and submits the payload to some external system, database or service for further processing.

![diagram](assets/diagram.png)

## Feature Status

- Data Types
  - CBOR blocks
  - CBOR txs
  - Oura v1 model (for backward-compatibility)
  - Parsed Txs (structured objects with all tx data)
  - Generic JSON (any kind of JSON values)
- Sources
  - chain-sync from local node
  - chain-sync + block-fetch from remote relay node
  - S3 bucket with block data
  - Kafka topic with block data 
- Sinks
  - Kafka topic
  - Elasticsearch index / data stream
  - Rotating log files with compression
  - Redis streams
  - AWS SQS queue
  - AWS Lambda call
  - AWS S3 objects
  - GCP PubSub
  - GCP Cloud Function
  - Azure Sinks
  - webhook (http post)
  - terminal (append-only, tail-like)
- Filters
  - Parse block / tx CBOR 
  - Split block into txs
  - Select Txs by matching rules (address, metadata, policies, etc)
  - Enrich tx data with related inputs
  - Custom Typescript code (uses Deno)
  - Custom WASM plugin
  - Rollback buffer with compensating actions
- Other
  - stateful chain cursor to recover from restarts
  - buffer stage to hold blocks until they reach a certain depth
  - pipeline metrics to track the progress and performance

## Known Limitations

- Oura reads events from minted blocks / transactions. Support for querying the mempool is not yet implemented.

## Contributing

All contributions are welcome, but please read the [contributing guide](.github/CONTRIBUTING.md#scope) of the project before starting to code.

## License

This project is licensed under the Apache-2.0 license. Please see the [LICENSE](LICENSE.md) file for more details.