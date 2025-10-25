# Metadata Regex Filter Example

Filter transactions by metadata content using regex patterns.

## Configuration

```toml
[[filters]]
type = "Select"
skip_uncertain = false

[filters.predicate.match.metadata]
label = 674

[filters.predicate.match.metadata.value.text]
regex = "testing regex"
```

## Running

```bash
oura daemon --config ./daemon.toml
```

## Features

- **Recursive search**: Automatically searches through nested arrays and maps
- **Flexible patterns**: Use standard regex syntax
- **Optional label**: Omit `label` field to search across all metadata

## Common Patterns

```toml
regex = "(?i)keyword"          # Case-insensitive
regex = "^MyApp:"               # Starts with
regex = "payment|donation"      # Multiple keywords
```

## See Also

- [Select Filter Documentation](../../docs/v2/filters/select.mdx)
- [CIP-20 Specification](https://cips.cardano.org/cips/cip20/)
