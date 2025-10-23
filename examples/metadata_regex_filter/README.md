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
skip_uncertain = true

[filters.predicate.match.tx]
metadata = [{
    label = 674,                              # CIP-20 message label
    value = { text = { regex = "(?i)hello.*world" } }  # Case-insensitive regex
}]
```

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
