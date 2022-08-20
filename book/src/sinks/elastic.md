# Elasticsearch

A sink that outputs events into an Elasticsearch server. Each event is json-encoded and sent as a message to an index or data stream.

## Configuration

```toml
[sink]
type = "Elastic"
url = "https://localhost:9200"
index = "oura.sink.v0.mainnet"

[sink.credentials]
type = "Basic"
username = "oura123"
password = "my very secret stuff"
```

### Section: `sink`

- `type`: the literal value `Elastic`.
- `url`: the location of the Elasticsearch's API
- `index`: the name of the index (or data stream) to store the event documents
- `idempotency` (optional): flag that if enabled will force idempotent calls to ES (to avoid duplicates)
- `retry_policy` (optional): controls the policy to retry failed requests (see [retry policy](../advanced/retry_policy.md))

### Section: `sink.credentials`

This section configures the auth mechanism against Elasticsearch. You can remove the whole section from the configuration if you want to skip authentication altogether (maybe private cluster without auth?).

We currently only implement _basic_ auth, other mechanisms will be implemented at some point.

- `type`: the mechanism to use for authentication, only `Basic` is currently implemented
- `username`: username of the user with access to Elasticsearch
- `password`: password of the user with access to Elasticsearch

## Idempotency

In services and API calls, _idempotency_ refers to a property of the system where the execution of multiple "equivalent" requests have the same effect as a single request. In other words, "idempotent" calls can be triggered multiple times without problem.

In our Elasticsearch sink, when the `idempotency` flag is enabled, each document sent to the index will specify a particular content-based ID: the [fingerprint](../filters/fingerprint.md) of the event. If Oura restarts without having a cursor or if the same block is processed for any reason, repeated events will present the same ID and Elasticsearch will reject them and Oura will continue with the following event. This mechanism provides a strong guarantee that our index won't contain duplicate data.

If the flag is disabled, each document will be generated using a random ID, ensuring that it will be indexed regardless.
