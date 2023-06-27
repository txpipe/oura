# Google Cloud Functions

A sink that sends each event to a cloud function. Each event is json-encoded and sent as a POST request.

## Configuration

```toml
[sink]
type = "GcpCloudFunction"
url = "https://REGION-PROJECT_ID.cloudfunctions.net/FUNCTION_NAME"
timeout = 30000
error_policy = "Continue"
authorization = "user:pass"

[sink.headers]
extra_header_1 = "abc"
extra_header_2 = "123"

[sink.retry_policy]
max_retries = 30
backoff_unit =  5000
backoff_factor = 2
max_backoff = 100000
```

### Section: `sink`

- `type`: the literal value `GcpCloudFunction`
- `url`: Your function url
- `timeout` (optional): the timeout value for the HTTP response in milliseconds. Default value is `30000`.
- `authorization` (optional): value to add as the 'Authorization' HTTP header
- `headers` (optional): key-value map of extra headers to pass in each HTTP call
- `error_policy` (optional): either `Continue` or `Exit`. Default value is `Exit`.
- [retry_policy](../advanced/retry_policy.md)
