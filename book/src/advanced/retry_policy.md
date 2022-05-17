# Retry Policy

Advanced options for instructing Oura how to deal with failed attempts in certain sinks.

## Supported Sinks

- [GCP CloudFunction](../sinks/gcp_cloudfunction.md)
- [GCP PubSub](../sinks/gcp_pubsub.md)
- [Webhook](../sinks/webhook.md)

## Configuration

To modify the default behaviour used by the sink, a section named `[sink.retry_policy]` needs to be added in the `daemon.toml` file.

```toml
[sink.retry_policy]
max_retries = 30
backoff_unit =  5000
backoff_factor = 2
max_backoff = 100000
```

- `max_retries`: the max number of retries before failing the whole pipeline. Default value is `20`
- `backoff_unit`: the delay expressed in milliseconds between each retry. Default value is `5000`.
- `backoff_factor`: the amount to increase the backoff delay after each attempt. Default value is `2`.
- `max_backoff`: the longest possible delay in milliseconds. Default value is `100000`
