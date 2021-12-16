# Node-to-Client

The Node-to-Client (N2C) source uses Ouroboros mini-protocols to connect to a local Cardano node through a unix socket bearer and fetches block data using the ChainSync mini-protocol instantiated to "full blocks".

## Configuration

The following snippet shows an example of how to setup a typical N2C source:

```toml
[source]
type = "N2C"
address = ["Unix", "/opt/cardano/cnode/sockets/node0.socket"]
magic = "mainnet"
```

Field definition:

- `type`: this field must be set to the literal value `N2C`
- `address`: a tuple describing the location of the socket
- `magic`: the magic of the network that the node is running (`mainnet`, `testnet` or a custom numeric value)
