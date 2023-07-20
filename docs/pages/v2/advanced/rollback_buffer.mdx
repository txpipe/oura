# Rollback Buffer

The "rollback buffer" feature provides a way to mitigate the impact of chain rollbacks in downstream stages of the data-processing pipeline.

## Context

Handling rollbacks in a persistent storage requires clearing the orphaned data / blocks before adding new records. The complexity of this process may vary by concrete storage engine, but it always has an associated impact on performance. In some scenarios, it might even be prohibitive to process events without a reasonable level of confidence about the immutability of the record.

Rollbacks occur frequently under normal conditions, but the chances of a block becoming orphaned diminishes as the depth of the block increases. Some Oura use-cases may benefit from this property, some pipelines might prefer lesser rollback events, even if it means waiting for a certain number of confirmations.

## Feature

Oura provides a "rollback buffer" that will hold blocks in memory until they reach a certain depth. Only blocks above a min depth threshold will be sent down the pipeline. If a rollback occurs and the intersection is within the scope of the buffer, the rollback operation will occur within memory, totally transparent to the subsequent stages of the pipeline.

If a rollback occurs and the intersection is outside of the scope of the buffer, Oura will fallback to the original behaviour and publish a RollbackEvent so that the "sink" stages may handle the rollback procedure manually.

## Trade-off

There's an obvious trade-off to this approach: latency. A pipeline will not process any events until the buffer fills up. Once the initial wait is over, the throughput of the whole pipeline should be equivalent to having no buffer at all (due to Oura's "pipelining" nature). If a rollback occurs, an extra delay will be required to fill the buffer again.

Notice that even if the throughput isn't affected, the latency (measured as the delta between the timestamp at which the event reaches the "sink" stage and the original timestamp of the block) will always be affected by a fixed value proportional to the size of the buffer.

## Implementation Details

The buffer logic is implemented in pallas-miniprotocols library. It works by keeping a VecDeque data structure of chain "points", where roll-forward operations accumulate at the end of the deque and retrieving confirmed points means popping from the front of the deque.

## Configuration

The min depth is a configurable setting available when running in daemon mode. Higher min_depth values will lower the chances of experiencing a rollback event, at the cost of adding more latency. An node-to-node source stage config would look like this:

```toml
[source]
type = "N2N"
address = ["Tcp", "relays-new.cardano-mainnet.iohk.io:3001"]
magic = "mainnet"
min_depth = 6
```

Node-to-client sources provide an equivalent setting.
