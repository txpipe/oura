# Google Cloud PubSub

A sink that sends each event as a message to a PubSub topic. Each event is json-encoded and sent to a configurable PubSub topic.

## Configuration

```toml
[sink]
type = "GcpPubSub"
credentials = "oura-test-347101-ff3f7b2d69cc.json"
topic = "test"
```

### Section: `sink`

- `type`: the literal value `GcpPubSub`.
- `credentials`: the service account json file downloaded from the cloud console.
- `topic`: the short name of the topic to send message to.
