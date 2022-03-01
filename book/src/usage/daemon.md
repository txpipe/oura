# Daemon Mode

_Oura's_ `daemon` mode processes data in the background, without any live output. This mode is used in scenarios where you need to continuosuly bridge blockchain data with other persistence mechanisms or to trigger an automated process in response to a particular event pattern.

## Start Daemon Mode

To start _Oura_ in _daemon mode_, use the following command:

```
oura dameon
```

By default, _Oura_ will load the configuration values from `/etc/oura/daemon.toml`.

If you need to specify a different configuration path, use the following command:

```sh
oura daemon --config my_config.toml
```

## Configuration

The configuration file needs to specify the source, filters and sink to use in a particular pipeline. The following toml represent the typical skeleton of an _Oura_ config file:

```toml
[source]
type = "X" # the type of source to use

# custom config fields for this source type
foo = "abc"
bar = "xyz"

[[filters]]
type = "Y" # the type of filter to use

# custom config fields for this filter type
foo = "123"
bar = "789"

[sink]
# the type of sink to use
type = "Z"

# custom config fields for this sink type
foo = "123"
bar = "789"

# optional cursor settings, remove seaction to disable feature
[cursor]
type = "File"
path = "/var/oura/cursor"
```

### The `source` section

This section specifies the origin of the data. The special `type` field must always be present and containing a value matching any of the available built-in sources. The rest of the fields in the section will depend on the selected `type`. See the [sources](../sources/index.md) section for a list of available options and their corresponding config values.

### The `filters` section

This section specifies a collection of filters that are applied in sequence to each event. The sepcial `type` field must always be present and containing a value matching any of the available built-in filters. Notice that this section of the config is an array, allowing multiple filter sections per config file. See the [filters](../filters/index.md) section for a list of available options and their corresponding config values.

### The `sink` section

This section specifies the destination of the data. The special `type` field must always be present and containing a value matching any of the available built-in sinks. The rest of the fields in the section will depend on the selected `type`. See the [sinks](../sinks/index.md) section for a list of available options.

### The `cursor` section

This section specifies how to configure the [cursor feature](../advanced/stateful_cursor.md). A cursor is a reference of the current position of the pipeline. If the pipeline needs to restart for whatever reason, and a cursor is available, the pipeline will start reading from that point in the chain. Removing the section from the config will disable the cursor feature.

### Full Example

Here's an example configuration file that uses a Node-to-Node source and output the events into a Kafka sink:

```toml
[source]
type = "N2N"
address = ["Tcp", "relays-new.cardano-mainnet.iohk.io:3001"]
magic = "mainnet"

[[filters]]
type = "Fingerprint"

[[filters]]
type = "Selection"
predicate = "variant_in"
argument = ["Block", "Transaction"]

[sink]
type = "Kafka"
brokers = ["127.0.0.1:53147"]
topic = "testnet"
```
