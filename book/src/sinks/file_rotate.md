# File Rotate

A sink that saves events into the file system. Each event is json-encoded and appended to the of a text file. Files are rotated once they reach a certain size. Optionally, old files can be automatically compressed once they have rotated.

## Configuration

Example sink section config

```toml
[sink]
type = "FileRotate"
output_path = "/var/oura/mainnet"
output_format = "JSONL"
max_bytes_per_file = 1_000_000
max_total_files = 10
compress_files = true
```

### Section: `sink`

- `type`: the literal value `FileRotate`.
- `output_path`: the path-like prefix for the output log files
- `output_format` (optional): specified the type of syntax to use for the serialization of the events. Only available option at the moment is `JSONL` (json + line break)
- `max_bytes_per_file` (optional): the max amount of bytes to add in a file before rotating it
- `max_total_files` (optional): the max amount of files to keep in the file system before start deleting the old ones
- `compress_files` (optional): a boolean indicating if the rotated files should be compressed.
