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

[sink.attributes]
network = "mainnet"
version = "1"
```

### Section: `sink`

- `type`: the literal value `GcpPubSub`.
- `topic`: the short name of the topic to send message to.
- `error_policy` (optional): either `Continue` or `Exit`. Default value is `Exit`.
- `ordering_key` (optional): the key to use for ordering messages. If not specified, messages will be sent in an unordered manner.
- `attributes` (optional): a map of attributes to add to each message. The key and value must be strings.
- [retry_policy](../advanced/retry_policy.md)

### GCP Authentication

The GCP authentication process relies on the following conventions:

- If the `GOOGLE_APPLICATION_CREDENTIALS` environmental variable is specified, the value will be used as the file path to retrieve the JSON file with the credentials.
- If the server is running on GCP, the credentials will be retrieved from the metadata server.
- If `PUBSUB_EMULATOR_HOST` environment variable is present, the sink will skip authentication and connect to the emulator instead of the production service.
