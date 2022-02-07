# Node-to-Client

The Node-to-Client (N2C) source uses Ouroboros mini-protocols to connect to a local Cardano node through a unix socket bearer and fetches block data using the ChainSync mini-protocol instantiated to "full blocks".

## Configuration

The following snippet shows an example of how to setup a typical N2C source:

```toml
[source]
type = "N2C"
address = ["Unix", "<socket location>"]
magic = <network magic>
since = [<slot>, "<block hash>"]

[source.mapper]
include_block_end_events = <bool>
include_transaction_details = <bool>
include_transaction_end_events = <bool>
include_block_cbor = <bool>
```

### Section `source`:

- `type`: this field must be set to the literal value `N2C`
- `address`: a tuple describing the location of the socket
- `magic`: the magic of the network that the node is running (`mainnet`, `testnet` or a custom numeric value)
- `since`: the point in the chain where reading of events should start from. It must be specified as a tuple of slot (integer) and block hash (hex string)

### Section `source.mapper`

This section provides options to tweak the behaviour of how raw chain data is mapped into _Oura_ events.

- `include_block_end_events`: instructs the mapper to include an event for when the mapper
  finishes crawling a block record.
- `include_transaction_details`: instructs the mapper to include all details in the transaction event payload (inputs, outputs, metadata, mint, etc)
- `include_transaction_end_events`: instructs the mapper to include an event for when the mapper
  finishes crawling a transaction record.
- `include_block_cbor`: instructs the mapper to include the hex of the cbor in the block record.

## Examples

Connecting to a local Cardano node in mainnet through unix sockets:

```toml
[source]
type = "N2C"
address = ["Unix", "/opt/cardano/cnode/sockets/node0.socket"]
magic = "mainnet"
```

Connecting to a local Cardano node in testnet through unix sockets:

```toml
[source]
type = "N2C"
address = ["Unix", "/opt/cardano/cnode/sockets/node0.socket"]
magic = "testnet"
```

Start reading from a particular point in the chain:

```toml
[source]
type = "N2C"
address = ["Unix", "/opt/cardano/cnode/sockets/node0.socket"]
magic = "mainnet"
since = [48896539, "5d1f1b6149b9e80e0ff44f442e0cab0b36437bb92eacf987384be479d4282357"]
```

Include all details inside the transaction events:

```toml
[source]
type = "N2C"
address = ["Unix", "/opt/cardano/cnode/sockets/node0.socket"]
magic = "mainnet"


[source.mapper]
include_transaction_details = true
include_block_cbor = true
```
