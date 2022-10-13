# Cardano => Kafka

This guide shows how to leverage _Oura_ to stream data from a Cardano node into a _Kafka_ topic.

## About Kafka

> Apache Kafka is a framework implementation of a software bus using stream-processing. It is an open-source software platform developed by the Apache Software Foundation written in Scala and Java.

Find [more info](https://en.wikipedia.org/wiki/Apache_Kafka) about _Kafka_ in wikipedia or visit _Kafka's_ official [website](https://kafka.apache.org/)


## Prerequisites

This examples assumes the following prerequisites:

- A running Cardano node locally accesible via a unix socket.
- A Kafka cluster accesible through the network.
- An already existing Kafka topic where to output events
- _Oura_ binary release installed in local system

## Instructions

### 1. Create an Oura configuration file `cardano2kafka.toml`

```toml
[source]
type = "N2C"
address = ["Unix", "/opt/cardano/cnode/sockets/node0.socket"]
magic = "testnet"

[sink]
type = "Kafka"
brokers = ["kafka-broker-0:9092"]
topic = "cardano-events"
```

Some comments regarding the above configuration:

- the `[source]` section indicates _Oura_ from where to pull chain data.
    - the `N2C` source type value tells _Oura_ to get the data from a Cardano node using Node-to-Client mini-protocols (chain-sync instantiated to full blocks).
    - the `address` field indicates that we should connect via `Unix` socket at the specified path. This value should match the location of your local node socket path.
    - the `magic` field indicates that our node is running in the `testnet` network. Change this value to `mainnet` if appropriate.
- the `[sink]` section tells _Oura_ where to send the information it gathered.
    - the `Kafka` sink type value indicates that _Oura_ should use a _Kafka_ producer client as the output
    - the `brokers` field indicates the location of the _Kafka_ brokers within the network. Several hostname:port pairs can be added to the array for a "cluster" scenario.
    - the `topic` fields indicates which _Kafka_ topic to used for the outbound messages.

### 2. Run _Oura_ in `daemon` mode

Run the following command from your terminal to start the daemon process:

```sh
RUST_LOG=info oura daemon --config cardano2kafka.toml
```

You should see an output similar to the following:

```
[2021-12-13T22:16:43Z INFO  oura::sources::n2n::setup] handshake output: Accepted(7, VersionData { network_magic: 764824073, initiator_and_responder_diffusion_mode: false })
[2021-12-13T22:16:43Z INFO  oura::sources::n2n::setup] chain point query output: Some(Tip(Point(47867448, "f170baa5702c91b23580291c3a184195df7c77d3e1a03b3d6424793aacc850d6"), 6624258))
[2021-12-13T22:16:43Z INFO  oura::sources::n2n::setup] node tip: Point(47867448,"f170baa5702c91b23580291c3a184195df7c77d3e1a03b3d6424793aacc850d6")
[2021-12-13T22:16:44Z INFO  oura::sources::n2n] rolling block to point Point(47867448, "f170baa5702c91b23580291c3a184195df7c77d3e1a03b3d6424793aacc850d6")
[2021-12-13T22:16:52Z INFO  oura::sources::n2n] requesting block fetch for point Some(Point(47867448, "f170baa5702c91b23580291c3a184195df7c77d3e1a03b3d6424793aacc850d6"))
[2021-12-13T22:17:15Z INFO  oura::sources::n2n] requesting block fetch for point Some(Point(47867448, "f170baa5702c91b23580291c3a184195df7c77d3e1a03b3d6424793aacc850d6"))
[2021-12-13T22:17:20Z INFO  oura::sources::n2n] requesting block fetch for point Some(Point(47867448, "f170baa5702c91b23580291c3a184195df7c77d3e1a03b3d6424793aacc850d6"))
```


