# Selection Filter

A filter that evaluates a set of configurable predicates against each event in the pipeline to decide which records should be sent to the following stage.

Not every use-case requires each and every event to be processed. For example, a pipeline interested in creating a 'metadata' search engine might not care about transaction outputs. With a similar logic, a pipeline aggregating transaction amounts might not care about metadata. The _selection filter_ provides a way to optimize the pipeline so that only relevant events are processed.

The filter works by evaluating a predicate against each event. If the predicate returns `true`, then the event will continue down the pipeline. If the predicate evalutes to `false`, the event will be dopped. We currently provide some common built-in predicate to facilitate common use-cases (eg: matching event type, matching policy id, matching a metadata key, etc.). We also provide some 'connecting' predicates like `all_of`, `any_of`, and `not` which can be used to create complex conditions by composing other predicates.

## Configuration

Adding the following section to the daemon config file will enable the filter as part of the pipeline:

```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "<predicate kind>"
argument = <predicate argument>
```

### Section: `filters`

- `type`: the literal value `Selection`.

### Section `filters.check`

- `predicate`: the key of the predicate to use for the evaluation. See the list of available predicates for possible values.
- `argument`: a polimorphic argument that specializes the behavior of the predicate in some way.

## Available Predicates

- `variant_in (string[])`: This predicate will yield true when the variant of the event matches any of the items in the argument array.
- `variant_not_in (string[])`: This predicate will yield true when the variant of the event doesn't match any of the items in the argument array.
- `policy_equals (string)`: This predicate will yield true when the policy of a mint or output asset matches the value in the argument.
- `asset_equals (string)`: This predicate will yield true when the policy of a mint or output asset matches the value in the argument.
- `metadata_label_equals (string)`: This predicate will yield true when the root label of a metadata entry matches the value in the argument.
- `metadata_any_sub_label_equals (string)`: This predicate will yield true when _at least one_ of the sub labels in a metadata entry matches the value in the argument.
- `not (predicate)`: This predicate will yield true when the predicate in the arguments yields false.
- `any_of (predicate[])`: This predicate will yield true when _any_ of the predicates in the argument yields true.
- `all_of (predicate[])`: This predicate will yield true when _all_ of the predicates in the argument yields true.

## Examples

Allowing only block and transaction events to pass:

```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "variant_in"
argument = ["Block", "Transaction"]
```

Using the `not` predicate to allow all events except the variant `Transaction`:

```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "not"

[filters.check.argument]
predicate = "variant_in"
argument = ["Transaction"]
```

Using the `any_of` predicate to filter events presenting any of two different policies (Boolean "or"):

```toml
[filters.check]
predicate = "any_of"

[[filters.check.argument]]
predicate = "policy_equals"
argument = "4bf184e01e0f163296ab253edd60774e2d34367d0e7b6cbc689b567d"

[[filters.check.argument]]
predicate = "policy_equals"
argument = "a5bb0e5bb275a573d744a021f9b3bff73595468e002755b447e01559"
```

Using the `all_of` predicate to filter only "asset" events presenting a particular policy (Boolean "and") :

```toml
[filters.check]
predicate = "all_of"

[[filters.check.argument]]
predicate = "variant_in"
argument = ["OutputAsset"]

[[filters.check.argument]]
predicate = "policy_equals"
argument = "a5bb0e5bb275a573d744a021f9b3bff73595468e002755b447e01559"
```
