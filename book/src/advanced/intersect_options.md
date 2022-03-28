# Intersect Options

Advanced options for instructing Oura from which point in the chain to start reading from.

## Feature

When running in daemon mode, Oura provides 4 different strategies for finding the intersection point within the chain sync process.

- `Origin`: Oura will start reading from the beginning of the chain.
- `Tip`: Oura will start reading from the current tip of the chain.
- `Point`: Oura will start reading from a particular point (slot, hash) in the chain. If the point is not found, the process will be terminated with a non-zero exit code.
- `Fallbacks`: Oura will start reading the first valid point within a set of alternative positions. If point is not valid, the process will fallback into the next available point in the list of options. If none of the points are valid, the process will be terminated with a non-zero exit code.

The default strategy use by Oura is `Tip`, unless an alternative option is specified via configuration.

## Configuration

To modify the default behaviour used by the daemon mode, a section named `[source.intersect]` needs to be added in the `daemon.toml` file.

```toml
[source.intersect]
type = <Type>
value = <Value>
```

- `type`: Defines which strategy to use. Valid values are `Origin`, `Tip`, `Point`, `Fallbacks`. Default value is `Tip`.
- `value`: Either a point or an array of points to be used as argument for the selected strategy.

## Examples

The following example show how to configure Oura to use a set of fallback intersection point. The chain sync process will attempt to first intersect at slot `4449598`. If not found, it will continue with slot `43159` and finally with slot `0`.

```toml
[source.intersect]
type = "Fallbacks"
value = [
    [4449598, "2c9ba2611c5d636ecdb3077fde754413c9d6141c6288109922790e53bbb938b5"],
    [43159, "f5d398d6f71a9578521b05c43a668b06b6103f94fcf8d844d4c0aa906704b7a6"],
    [0, "f0f7892b5c333cffc4b3c4344de48af4cc63f55e44936196f365a9ef2244134f"],
]
```
