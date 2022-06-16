# Google Cloud PubSub

A sink that sends each event as a message to a PubSub topic. Each event is json-encoded and sent to a configurable PubSub topic.

## Configuration

```toml
[sink]
type = "GcpPubSub"
credentials = "oura-test-347101-ff3f7b2d69cc.json"
topic = "test"

[sink.retry_policy]
max_retries = 30
backoff_unit =  5000
backoff_factor = 2
max_backoff = 100000
```

### Section: `sink`

- `type`: the literal value `GcpPubSub`.
- `credentials`: the path to the service account json file downloaded from the cloud console.
- `topic`: the short name of the topic to send message to.
- `error_policy` (optional): either `Continue` or `Exit`. Default value is `Exit`.
- [retry_policy](../advanced/retry_policy.md)
