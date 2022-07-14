# Redis Streams

A sink that outputs events into _Redis Stream_.

_Redis Streams_ works as an append-only log where multiple consumers can read from the same queue while keeping independent offsets (as opposed to a PubSub topic where one subscriber affect the other). You can learn more about the _Streams_ feature in the official [Redis Documentation](https://redis.io/docs/manual/data-types/streams).

This sink will process incoming events and send a JSON-encoded message of the payload for each one using the `XADD` command. The Redis instance can be local or remote.

## Configuration

Example configuration that sends all events into a single stream named `mystream` of a Redis instance running in port 6379 of the localhost.

```toml
[sink]
type = "Redis"
redis_server = "redis://localhost:6379"
stream_name = "mystream"
```

Example configuration that sends events into different streams (named by the type of event) of a Redis instance running in port 6379 of the localhost.

```toml
[sink]
type = "Redis"
redis_server = "redis://localhost:6379"
stream_strategy = "ByEventType"
```

### Section: `sink`

- `type`: the literal value `Redis`.
- `redis_server`: the redis server in the format `redis://[<username>][:<password>]@<hostname>[:port][/<db>]`
- `stream_name` : the name of the redis stream for StreamStrategy `None`, default is "oura" if not specified
- `stream_strategy` : `None` or `ByEventType`

## Conventions

It is possible to send all Event to a single stream or create multiple streams, one for each event type. By appling the [selection](/filters/selection) filter it is possible to define the streams which should be created. 

The sink uses the default Redis convention to define the unique entry ID for each message sent to the stream ( `<millisecondsTime>-<sequenceNumber>`).

Messages in Redis Streams are required to be `hashes` (maps between the string fields and the string values). This sink will serialize the event into a single-entry map with the following parts:

- `key`: the [fingerprint](/filters/fingerprint) value if available, or the event type name.
- `value`: the json-encoded payload of the event.
