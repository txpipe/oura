# Dump Mode

The `dump` mode provides a quick way to tail the latest events from the blockchain and outputs raw data into stdout or the file system. It connects directly to a Cardano node using either node-to-client or node-to-node protocols. The output is formatted using JSONL (json, one-line per event). This command is intended mainly as quick persistence mechanism of blockchain data, such as keeping a log of blocks / transactions. It can also be used for "piping" stdout into other shell commands.

If an output path is specified, data will be saved as a set of rotation logs files. Each log file will contain a max of ~50mb. A total of 200 files will be stored before starting to delete the older ones. Gzip is used to compress old files as they are rotated.

For real-time, human-inspection of the events, use the [watch command](watch.md).

## Usage

To start _Oura_ in dump mode, use the following command from your shell:

```
oura dump [OPTIONS] <socket>
```

- `<socket>`: this a required argument that specifies how to connect to the cardano node. It can either be a tcp address (`<host>:<port>` syntax) or a file path pointing to the location of the unix socket.

### Options

- `--bearer <bearer>`: an option that specifies the type of bearer to use. Possible values are `tcp` and `unix`. If omitted, the value `unix` is used as default.
- `--magic <magic>`: the magic number of the network you're connecting to. Possible values are `mainnet`, `testnet` or a numeric value. If omitted, the value `mainnet` is used as default.
- `--mode <mode>`: an option to force the which set of mini-protocols to use when connecting to the Cardano node. Possible values: `node` and `client`.  If omitted, _Oura_ will infer the standard protocols for the specified bearer.
- `--since <slot>,<hash>`: an option to specify from which point in the chain _Oura_ should start reading from. The point is referenced by passing the slot of the block followed by a comma and the hash of the block (`<slot>,<hash>`). If omitted, _Oura_ will start reading from the tail (tip) of the node.
- `--output <path-like>`: an option to specify an output file prefix for storing the log files. Logs are rotated, so a timestamp will be added as a suffix to the final filename. If omitted, data will be sent to stdout.

## Examples

### Dump Data From A Remote Relay Node into Stdout

```sh
oura dump relays-new.cardano-mainnet.iohk.io:3001 --bearer tcp
```

### Dump Data From A Remote Relay Node into Rotating Files

```sh
oura dump relays-new.cardano-mainnet.iohk.io:3001 --bearer tcp --output ./mainnet-logs
```

### Pipe Data From A Remote Relay Node into a new Shell Command

```sh
oura dump relays-new.cardano-mainnet.iohk.io:3001 --bearer tcp | grep block
```
