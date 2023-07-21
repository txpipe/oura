# Mapper Options

A set of "expensive" event mapping procedures that require an explicit opt-in to be activated.

## Context

One of the main concerns of Oura is turning block / tx data into atomic events to send down the pipeline for further processing. The `source` stage is responsible for executing these mapping procedures.

Most of the time, this logic is generic enough that it can be reused in different scenarios. For example, the `N2N` and the `N2C` sources share the same mapping procedures. If a particular use-case needs to cherry-pick, enrich or alter the data in some way, the recommendation is to handle the transformation in downstream stages, by using any of the built-in filter or by creating new ones.

There are some exceptions though, whenever a mapping has a heavy impact on performance, it is better to disable it completely at the `source` level to avoid paying the overhead associated with the initial processing of the data.

## Feature

We consider a mapping procedure "expensive" if it involves: handling a relative large amount of data, computing some relatively expensive value or generating redundant data required only for very particular use cases.

For these expensive procedures, we provide configurable options that instructs an Oura instance running in daemon mode to opt-in on each particular rule.

## Configuration

The mapper options can be defined by adding the following configuration in the `daemon.toml` file:

```toml
[source.mapper]
include_block_end_events = <bool>
include_transaction_details = <bool>
include_transaction_end_events = <bool>
include_block_cbor = <bool>
include_byron_ebb = <bool>
```

- `include_block_end_events`: if enabled, the source will output an event signaling the end of a block, duplicating all of the data already sent in the corresponding block start event. Default value is `false`.
- `include_transaction_details`: if enabled, each transaction event payload will contain an nested version of all of the details of the transaction (inputs, outputs, mint, assets, metadata, etc). Useful when the pipeline needs to process the tx as a unit, instead of handling each sub-object as an independent event. Default value is `false`.
- `include_transaction_end_events`: if enabled, the source will output an event signaling the end of a transaction, duplicating all of the data already sent in the corresponding transaction start event. Defaul value is `false`.
- `include_block_cbor`: if enabled, the block event will include the raw, unaltered cbor content received from the node, formatted as an hex string. Useful when some custom cbor decoding is required. Default value is `false`.
- `include_byron_ebb`: if enabled, a block event will be emmitted for legacy epoch boundary block of the Byron era (deprecated in newer eras). Useful when performing validation on previous block hashes. Default value is `false`.
