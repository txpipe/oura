# Redis Streams

A sink that implements a _Redis Stream_ producer. The sink allows different stream strategies.

It is possible to send all Event to a single stream or create multiple streams, one for each event type. 
Both modes use `<millisecondsTime>-<sequenceNumber>` as unique entry ID (redis stream standard).
With StreamStrategy `None` a single redis-stream is used for all events, a stream name can be defined by `stream_name`, the default stream name is `oura`. 
StreamStrategy `ByEventType` creates its own redis-stream for each event type. By appling filters it is possible to define the streams which should be created. 

The sink will use fingerprints as keys, if fingerprints are active otherwise the event type name in lowercase is used.

## Configuration

_Single Stream Mode:_

```toml
[sink]
type = "Redis"
redis_server = "redis://default:@127.0.0.1:6379/0"
stream_name = "mystream"
stream_strategy = "None"
```

_Multi Stream Mode:_
```toml
[sink]
type = "Redis"
redis_server = "redis://default:@127.0.0.1:6379/0"
stream_strategy = "ByEventType"
```

- `type`: the literal value `Redis`.
- `redis_server`: the redis server in the format `redis://[<username>][:<password>]@<hostname>[:port][/<db>]`
- `stream_name` : the name of the redis stream for StreamStrategy `None`, default is "oura" if not specified
- `stream_strategy` : `None` or `ByEventType`