# Terminal

A sink that outputs each event into the terminal through stdout using fancy coloring 💅.

## Configuration

```toml
[sink]
type = "Terminal"
throttle_min_span_millis = 500
```

- `type` (required): the literal value `Terminal`.
- `throttle_min_span_millis` (optional, default = `500`): the amount of time (milliseconds) to wait between printing each event into the console. This is used to facilitate the reading for human following the output.
