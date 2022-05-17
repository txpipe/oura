# Webhook

A sink that outputs each event as an HTTP call to a remote endpoint. Each event is json-encoded and sent as the body of a request using `POST` method.

The sink expect a 200 reponse code for each HTTP call. If found, the process will continue with the next message. If an error occurs (either at the tcp or http level), the sink will apply the corresponding retry logic as specified in the configuration.

## Configuration

```toml
[sink]
type = "Webhook"
url = "https://endpoint:5000/events"
authorization = "user:pass"
timeout = 30000
error_policy = "Continue"

[sink.retry_policy]
max_retries = 30
backoff_unit =  5000
backoff_factor = 2
max_backoff = 100000

[sink.headers]
extra_header_1 = "abc"
extra_header_2 = "123"
```

### Section: `sink`

- `type`: the literal value `Webhook`.
- `url`: url of your remote endpoint (needs to accept POST method)
- `authorization` (optional): value to add as the 'Authorization' HTTP header
- `headers` (optional): key-value map of extra headers to pass in each HTTP call
- `timeout` (optional): the timeout value for the HTTP response in milliseconds. Default value is `30000`.
- `error_policy` (optional): either `Continue` or `Exit`. Default value is `Exit`.
- [retry_policy](../advanced/retry_policy.md)
