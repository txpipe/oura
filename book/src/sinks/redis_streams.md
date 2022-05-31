# Redis Streams

A sink that implements a _Redis Stream_ producer. The sink allows different configurations.

It is possible to send all Event to a single stream or create multiple streams, one for each event type. 
Both modes use `<millisecondsTime>-<sequenceNumber>` as unique entry ID (redis stream standard).
In SingleStream mode the Event-Type is used as key and the stream can be named by `stream_name`, the default stream name is `oura`. 
In MultiStream the Event-Type is used as name of the stream and the keys are created as followed:

Event-Type (stream)
Block               -> block-number
BlockEnd            -> block-number
TxInput             -> txhash#index
TxOutput            -> txhash#index
OutputAsses         -> policyId.AssetName
Collateral          -> txhash#index
StakeDelegation     -> pool-hash
RollBack            -> block-hash

All other Events    -> tx-hash

The sink will use fingerprints as keys, if fingerprints are active.

## Configuration

_Single Stream Mode:_

```toml
[sink]
type = "Redis"
redis_server = "redis://default:@127.0.0.1:6379/0"
stream_name = "mystream"
stream_config = "SingleStream"
```

_Multi Stream Mode:_
```toml
[sink]
type = "Redis"
redis_server = "redis://default:@127.0.0.1:6379/0"
stream_config = "MultiStream"
```

- `type`: the literal value `Redis`.
- `redis_server`: the redis server in the format `redis://[<username>][:<password>]@<hostname>[:port][/<db>]`
- `stream_name` : the name of the stream in SingleStream mode, default is "oura" if not specified
- `stream_config` : "SingleStream" or "MultiStream" to select the mode