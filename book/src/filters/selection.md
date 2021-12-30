# Selection Filter

A filter that evaluates a set of configurable predicates against each event in the pipeline to decide which records should be sent to the following stage. 

Not every use-case requires each and every event to be processed. For example, a pipeline interested in creating a 'metadata' search engine might not care about transactions outputs. With a similar logic, a pipeline aggregating transaction amounts might not care about metadata. The _selection filter_ provides a way to optimize the pipeline so that only relevant events are processed.

The filter works by evaluating a predicate against each event. If the predicate returns `true`, then the event will continue down the pipeline. If the predicate evalutes to `false`, the event will be dopped. We currently provide some common built-in predicate to facilitate common use-cases (eg: matching event type, matching policy id, matching a metadata key, etc). We also include some 'connecting' predicates (and / or / not) which can be used to create complex conditions by composing other predicates.

## Available Predicates

- `variant_in (string[])`: This predicate will yield true when the variant of the event matches any of the items in the argument array.
- `variant_not_in (string[])`: This predicate will yield true when the variant of the event doesn't match any of the items in the argument array.
- `policy_equals (string)`: This predicate will yield true when the policy of a mint or output asset matches the value in the argument.
- `asset_equals (string)`: This predicate will yield true when the policy of a mint or output asset matches the value in the argument.
- `metadata_key_equals (string)`: This predicate will yield true when the root key of a metadata entry matches the value in the argument.
- `metadata_subkey_equals (string)`: This predicate will yield true when any of the sub key in a metadata entry matches the value in the argument.
- `not (predicate)`: This predicate will yield true when the predicate in the arguments yields false.
- `any_of (predicate[])`: This predicate will yield true when _any_ of the predicates in the argument yields true.
- `all_of (predicate[])`: This predicate will yield true when _all_ of the predicates in the argument yields true.

## Configuration

Adding the following section to the daemon config file will enable the filter as part of the pipeline:

```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "<predicate kind>"
argument = "<predicate argument>"
```

### Section: `filters`

- `type`: the literal value `Selection`.

### Section `filters.check`

- `predicate`: the key of the predicate to use for the evaluation. See #predicate_types for more available options.
- `argument`: the


## Examples

Allowing only block and transaction events to pass:

```
[[filters]]
type = "Selection"

[filters.check]
predicate = "variant_in"
argument = ["Block", "Transaction"]
```

Using the `not` predicate to allow all events except the variant `Transaction`:

```
[[filters]]
type = "Selection"

[filters.check]
predicate = "not"

[filters.check.argument]
predicate = "variant_in"
argument = ["Transaction"]
```