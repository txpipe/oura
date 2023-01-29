# Google Cloud PubSub

A sink that sends each event as a message to a PubSub topic. Each event is json-encoded and sent to a configurable PubSub topic.

## Configuration

```toml
[sink]
type = "GcpPubSub"
topic = "test"

[sink.retry_policy]
max_retries = 30
backoff_unit =  5000
backoff_factor = 2
max_backoff = 100000
```

### Section: `sink`

- `type`: the literal value `GcpPubSub`.
- `topic`: the short name of the topic to send message to.
- `error_policy` (optional): either `Continue` or `Exit`. Default value is `Exit`.
- [retry_policy](../advanced/retry_policy.md)

### GCP Authentication

The GCP authentication process relies on the following conventions:

- If the `GOOGLE_APPLICATION_CREDENTIALS` environmental variable is specified, the value will be used as the file path to retrieve the JSON file with the credentials.
- If the server is running on GCP, the credentials will be retrieved from the metadata server.
