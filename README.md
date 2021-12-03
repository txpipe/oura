<div align="center">
    <img src="assets/logo.svg" alt="Oura" width="500">  
</div>
<hr />

<h2 align="center">The tail of Cardano</h2>

<hr/>

## Introduction

We have tools to "explore" the Cardano blockchain, which are useful when you know what you're looking for. We argue that there's a different, complementary use-case which is to "observe" the blockchain and react to particular event patterns.

_Oura_ is a rust-native implementation of a pipeline that connects to the tip of a Cardano node through a combination of _Ouroboros_ mini-protocol (using either a unix socket or tcp bearer), filters the events that match a particular pattern and then submits a succint, self-contained payload to pluggable observers called "sinks".

## Etymology

The name of the tool is inspired by the `tail` command available in unix-like systems which is used to display the tail end of a text file or piped data. Cardano's consensus procotol name, _Ouroboros_, is a reference to the ancient symbol depicting a serpent or dragon eating its own tail, which means "tail eating". "Oura" is the ancient greek word for "tail".

## Features

- Sources
    - [x] chain-sync full-block (node-to-client)
    - [ ] chain-sync headers-only (node-to-node)
    - [ ] chain-sync + block-fetch (node-to-node)
    - [ ] shared file system
- Sinks
    - [ ] Kafka topic
    - [ ] Redis streams
    - [ ] AWS SQS queue
    - [ ] GCP PubSub
    - [ ] webhook (http post)
    - [ ] email
    - [x] terminal (append-only, tail-like)
    - [ ] TUI
- Filters
    - [ ] by event type (block, tx, mint, cert, etc)
    - [ ] by block property (size, tx count)
    - [ ] by tx property (fee, has native script, has plutus script, etc)
    - [ ] by utxo property (address, asset, amount range)
- Enrichment
    - [ ] policy info from metadata service
    - [ ] input tx info from Blockfrost api
    - [ ] address translation from ADAHandle

## Terminal Output Demo

In this terminal recording we get to see a few mins of live output from a testnet node connected to the terminal sink.

[![asciicast](https://asciinema.org/a/66x3QUjQm6KtCkPYREiBycR6b.svg)](https://asciinema.org/a/66x3QUjQm6KtCkPYREiBycR6b)

## Under the Hood

All the heavy lifting required to communicate with the Cardano node is done by the [Pallas](https://github.com/txpipe/pallas) library, which provides an implementation of the Ouroboros multiplixer and a few of the required mini-protocol state-machines (ChainSync and LocalState in particular).

The data pipeline makes heavy use (maybe a bit too much) of multi-threading and mpsc channels provided by Rust's `std::sync` library.