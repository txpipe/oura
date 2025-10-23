# Metadata Regex Filter Example

This example demonstrates how to filter Cardano transactions by metadata content using regular expressions.

## Overview

The configuration filters transactions that contain CIP-20 messages (label 674) matching a specific regex pattern. This is useful for:

- Monitoring specific message patterns on-chain
- Filtering application-specific messages
- Building notification systems for particular message content

## Configuration

The example uses the `Select` filter with a regex pattern for metadata text values:

```toml
[[filters]]
type = "Select"
skip_uncertain = false

[filters.predicate.match.metadata]
label = 674

[filters.predicate.match.metadata.value.text]
regex = "testing regex"
```

### How it works

- **Recursive search**: The pattern automatically searches through nested structures (arrays, maps)
- **Substring match**: `regex = "testing regex"` matches if the text contains "testing regex" anywhere
- **Case-sensitive by default**: Use `(?i)` prefix for case-insensitive matching

## Regex Pattern Examples

### Case-insensitive matching
```toml
value = { text = { regex = "(?i)pattern" } }
```

### Match specific words
```toml
value = { text = { regex = "\\b(urgent|important)\\b" } }
```

### Match URLs
```toml
value = { text = { regex = "https?://[^\\s]+" } }
```

### Match JSON-like structures
```toml
value = { text = { regex = "\\{.*\"action\".*\\}" } }
```

## Running the Example

```bash
oura daemon --config ./daemon.toml
```

### Example Output

When a matching transaction is found, Oura outputs:

```json
{
  "event": "apply",
  "point": {
    "hash": "ed6e69806786b4d51044f66671690d6290e426aa6361d0ddc45a3b6c2b0015c2",
    "slot": 105563197
  },
  "record": {
    "auxiliary": {
      "metadata": [
        {
          "label": "674",
          "value": {
            "map": {
              "pairs": [
                {
                  "key": {"text": "msg"},
                  "value": {
                    "array": {
                      "items": [{"text": "testing regex"}]
                    }
                  }
                }
              ]
            }
          }
        }
      ]
    },
    "hash": "Cci9gZHpdWf93uK/hQ6xIokOl5X+Do7ttMdu+en4wnw=",
    "fee": "171045",
    "successful": true
  }
}
```

✅ **Verified on preprod testnet** - Successfully filtered transactions with metadata label 674 containing "testing regex"

## Use Cases

1. **Application-specific messages**: Filter messages from your dApp
2. **Notification triggers**: Watch for specific keywords or patterns
3. **Data extraction**: Find transactions with structured data in metadata
4. **Compliance monitoring**: Track specific message patterns for audit purposes

## Metadata Labels

Common metadata labels:
- **674**: CIP-20 transaction messages
- **721**: CIP-25 NFT metadata
- **127**: Custom application data
- Custom labels: Use any number for your application

## See Also

- [CIP-20 Specification](https://cips.cardano.org/cips/cip20/)
- [Select Filter Documentation](../../docs/v2/filters/select.mdx)
