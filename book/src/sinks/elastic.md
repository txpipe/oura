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

### Section: `sink.credentials`

This section configures the auth mechanism against Elasticsearch. You can remove the whole section from the configuration if you want to skip authentication altogether (maybe private cluster without auth?).

We currently only implement _basic_ auth, other mechanisms will be implemented at some point.

- `type`: the mechanism to use for authentication, only `Basic` is currently implemented
- `username`: username of the user with access to Elasticsearch
- `password`: password of the user with access to Elasticsearch
