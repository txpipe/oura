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
    - [ ] kafka topic
    - [ ] redis streams
    - [ ] aws sqs queue
    - [ ] gcp pubsub
    - [ ] webhook (http post)
    - [ ] email (http post)
    - [x] terminal (append-only, tail-like)
    - [ ] TUI
- Filters
    - [ ] by event type (block, tx, mint, cert, etc)
    - [ ] by block property (size, tx count)