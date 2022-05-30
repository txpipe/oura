# Watch Mode

The `watch` mode provides a quick way to tail the latest events from the blockchain. It connects directly to a Cardano node using either node-to-client or node-to-node protocols. The output is sent into the terminal in a human-readable fashion.

The output is colorized by type of event and dynamically truncated to fit the width of the terminal. The speed of the output lines is throttled to facilitate visual inspection of each even, otherwise, all events for a block would be output simultaneously.

## Usage

To start _Oura_ in watch mode, use the following command from your terminal:

```
oura watch [OPTIONS] <socket>
```

- `<socket>`: this a required argument that specifies how to connect to the cardano node. It can either be a tcp address (`<host>:<port>` syntax) or a file path pointing to the location of the unix socket.

### Options

- `--bearer <bearer>`: an option that specifies the type of bearer to use. Possible values are `tcp` and `unix`. If omitted, the value `unix` is used as default.
- `--magic <magic>`: the magic number of the network you're connecting to. Possible values are `mainnet`, `testnet` or a numeric value. If omitted, the value `mainnet` is used as default.
- `--mode <mode>`: an option to force the which set of mini-protocols to use when connecting to the Cardano node. Possible values: `node` and `client`.  If omitted, _Oura_ will infer the standard protocols for the specified bearer.
- `--since <slot>,<hash>`: an option to specify from which point in the chain _Oura_ should start reading from. The point is referenced by passing the slot of the block followed by a comma and the hash of the block (`<slot>,<hash>`). If omitted, _Oura_ will start reading from the tail (tip) of the node.


## Examples

### Watch Live Data From A Remote Relay Node

```sh
oura watch relays-new.cardano-mainnet.iohk.io:3001 --bearer tcp
```

### Watch Live Data From A Local Node Via Unix Socket

```sh
oura watch /opt/cardano/cnode/sockets/node0.socket --bearer unix
```

### Watch Live Data From The Tip Of A Local Testnet Node

```sh
oura watch /opt/cardano/cnode/sockets/node0.socket --bearer unix --magic testnet
```

### Watch Data Starting At A Particular Block

```sh
oura watch relays-new.cardano-mainnet.iohk.io:3001 \
    --bearer tcp \
    --since 49159253,d034a2d0e4c3076f57368ed59319010c265718f0923057f8ff914a3b6bfd1314
```
