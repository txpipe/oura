# Google Cloud PubSub

A sink that sends each event as a message to a PubSub topic. Each event is json-encoded and sent to a configurable PubSub topic.

## Configuration

```toml
[sink]
type = "GcpPubSub"
topic = "test"
```

### Section: `sink`

- `type`: the literal value `GcpPubSub`.
- `topic`: the short name of the topic to send message to.

### GCP Authentication

The GCP authentication process relies on the following conventions:

- If the `GOOGLE_APPLICATION_CREDENTIALS` environmental variable is specified, the value will be used as the file path to retrieve the JSON file with the credentials.
- If the server is running on GCP, the credentials will be retrieved from the metadata server.
