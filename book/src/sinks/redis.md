# Redis Stream

A sink that implements a _Redis Stream_ producer. Each event is json-encoded and sent as a message to a named stream.

## Configuration

```toml
[sink]
type = "Redis"
url = "redis://127.0.0.1/"
stream = "mystream"
```

- `type`: the literal value `Redis`.
- `url`: the redis connection URL
- `stream` this field indicates which _Redis Stream_ to write to when sending outbound messages.
