# Node-to-Node

The Node-to-Node (N2N) source uses Ouroboros mini-protocols to connect to a local or remote Cardano node through a tcp socket bearer and fetches block data using the ChainSync mini-protocol instantiated to "headers only" and the BlockFetch mini-protocol for retrieval of the actual block payload.

## Configuration

The following snippet shows an example of how to setup a typical N2N source:

```toml
[source]
type = "N2N"
address = ["Unix", "relays-new.cardano-mainnet.iohk.io:3001"]
magic = "mainnet"
```

Field definition:

- `type`: this field must be set to the literal value `N2N`
- `address`: a tuple describing the location of the tcp endpoint
- `magic`: the magic of the network that the node is running (`mainnet`, `testnet` or a custom numeric value)
