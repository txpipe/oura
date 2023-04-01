### Table of contents
- [Selection filter](#selection-filter)
- [General predicates](#general-predicates)
- [Variant-restricted predicates](#variant-restricted-predicates)
- [Real world example](#real-world-example)
# Selection filter

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

## General predicates
Predicates available for events of any type.

### `variant_in (string[])`
 This predicate will yield true when the variant of the event matches any of the items in the argument array. Available variants:
- [Block](../reference/data_dictionary.md#block-event)
- [CIP15Asset](../reference/data_dictionary.md#cip15asset-event)
- [CIP25Asset](../reference/data_dictionary.md#cip25asset-event)
- [Collateral](../reference/data_dictionary.md#collateral-event)
- [GenesisKeyDelegation](../reference/data_dictionary.md#genesiskeydelegation-event)
- [Metadata](../reference/data_dictionary.md#metadata-event)
- [Mint](../reference/data_dictionary.md#mint-event)
- [MoveInstantaneousRewardsCert](../reference/data_dictionary.md#moveinstantaneousrewardscert-event)
- [NativeScript](../reference/data_dictionary.md#nativescript-event)
- [NativeWitness](../reference/data_dictionary.md#nativewitness-event)
- [OutputAsset](../reference/data_dictionary.md#outputasset-event)
- [PlutusDatum](../reference/data_dictionary.md#plutusdatum-event)
- [PlutusRedeemer](../reference/data_dictionary.md#plutusredeemer-event)
- [PlutusScript](../reference/data_dictionary.md#plutusscript-event)
- [PlutusWitness](../reference/data_dictionary.md#plutuswitness-event)
- [PoolRegistration](../reference/data_dictionary.md#poolregistration-event)
- [PoolRetirement](../reference/data_dictionary.md#poolretirement-event)
- [RollBack](../reference/data_dictionary.md#rollback-event)
- [StakeDelegation](../reference/data_dictionary.md#stakedelegation-event)
- [StakeDeregistration](../reference/data_dictionary.md#stakederegistration-event)
- [StakeRegistration](../reference/data_dictionary.md#stakeregistration-event)
- [Transaction](../reference/data_dictionary.md#transaction-event)
- [TxInput](../reference/data_dictionary.md#txinput-event)
- [TxOutput](../reference/data_dictionary.md#txoutput-event)
- [VKeyWitness](../reference/data_dictionary.md#vkeywitness-event)

**Example** - Allowing only block and transaction events to pass:

```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "variant_in"
argument = ["Block", "Transaction"]
```

### `variant_not_in (string[])`
 This predicate will yield true when the variant of the event doesn't match any of the items in the argument array.

**Example** - Allowing all events except transaction to pass:
```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "variant_not_in"
argument = ["Transaction"]
```

### `not (predicate)`
 This predicate will yield true when the predicate in the arguments yields false.


**Example** - Using the `not` predicate to allow all events except the variant `Transaction`:

```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "not"

[filters.check.argument]
predicate = "variant_in"
argument = ["Transaction"]
```

### `any_of (predicate[])`
 This predicate will yield true when _any_ of the predicates in the argument yields true.


**Example** - Using the `any_of` predicate to filter events presenting any of two different policies (Boolean "or"):

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

### `all_of (predicate[])`
 This predicate will yield true when _all_ of the predicates in the argument yields true.

**Example** - Using the `all_of` predicate to filter only "asset" events presenting a particular policy (Boolean "and") :

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

## Variant-restricted predicates
Predicates operating on a subset of event variants.


### `policy_equals (string)`
 This predicate will yield true when the policy of a mint or output asset matches the value in the argument.


**Variants:** `Transaction`, `Mint`, `CIP25Asset`, `OutputAsset`

**Example**
```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "policy_equals"
argument = "<policy_id>"
```

### `asset_equals (string)`
 This predicate will yield true when the asset (token name) of a mint or output asset matches the value in the argument.

**Variants:** `CIP25Asset`, `Transaction`, `OutputAsset`, `Mint`

**Example**
```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "asset_equals"
argument = "<asset>"
```

### `metadata_label_equals (string)`
 This predicate will yield true when the root label of a metadata entry matches the value in the argument.

**Variants:** `Metadata`, `Transaction`

**Example**
```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "metadata_label_equals"
argument = "<label>"
```

### `metadata_any_sub_label_equals (string)`
 This predicate will yield true when _at least one_ of the sub labels (keys in the json map) of a metadata entry matches the value in the argument.

**Variants:** `Metadata`

**Example**

```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "metadata_any_sub_label_equals"
argument = "<label>"
```

### `v_key_witnesses_includes (string)`
This predicate will yield true when at least one of the vkeys matches the value in the argument. This filter needs `include_transaction_details = true` to work.


**Variants:** `VKeyWitness`, `Transaction`

**Example**
``` toml
[source.mapper]
include_transaction_details = true

[[filters]]
type = "Selection"

[filters.check]
predicate = "v_key_witnesses_includes"
argument = "<vkey>"
```

## Real world example

Example using nested filters. Use `toml2json ./oura.toml | jq` to visualize the structure as json.

```toml
[[filters]]
type = "Selection"

[filters.check]
predicate = "any_of"

[[filters.check.argument]]
predicate = "variant_in"
argument = ["RollBack"]

[[filters.check.argument]]
predicate = "all_of"

[[filters.check.argument.argument]]
predicate = "variant_in"
argument = ["CIP25Asset"]

[[filters.check.argument.argument]]
predicate = "any_of"

[[filters.check.argument.argument.argument]]
predicate = "policy_equals"
argument = "<policy_a>"

[[filters.check.argument.argument.argument]]
predicate = "policy_equals"
argument = "<policy_b>"

[[filters.check.argument]]
predicate = "all_of"

[[filters.check.argument.argument]]
predicate = "variant_in"
argument = ["Transaction"]

[[filters.check.argument.argument]]
predicate = "v_key_witnesses_includes"
argument = "<vkey>"
```
