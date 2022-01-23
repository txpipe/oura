# Contributing

## Ways to Contribute

_ALL_ contributions are welcome: 

- if you want to report a bug, please create a github issue following some [basic guidelines](#bug-report).
- if you want request a feature, please create a github issue following some [basic guidelines](#feature-request).
- if you want to submit a bugfix or new feature, please create a [pull request](#pull-requests).

## License

This project is licensed under the Apache-2.0 license. 

All contributions to the project will be licensed under the same terms. Please make sure you agree with these terms before working on your contribution.

Please see the [LICENSE](LICENSE.md) file for more details.

## Scope of the Project

The architecture of _Oura_ is extensible by design. There's no strict bound to the set of sources, filters and sinks that can be included. One might be tempted to add new "plugins" for each particular use-case, but this would eventually turn the codebase unmaintanble.

To avoid an unbounded growth, we try to keep the set of built-in plugins limited only to general-purpose use-cases. If you have a very particular need, our recommendation is to create a new crate, add _Oura_ as a dependency and fill the gaps required for your use-case.

As a rule of thumb, any feature within the categories of the following list is within the scope and contributions on the subject will likely be included in the codebase:

- Sources that provide on-chain data (blocks, transactions, metadata, etc)
- Sources that provide mempool data (pending transactions)
- Sinks that output to well-known event-processing platforms (eg: Kafka)
- Sinks that output to well-known message-queues platforms (eg: RabbitMQ)
- Sinks that output to well-known no-sql platforms (eg: Elasticsearch)
- Sinks that output to well-known FaaS platforms (eg: AWS Lambda)
- Filters that enrich event data with well-known external metadata

## Bug Report

Please follow these basic guidelines when reporting a bug:

- Create a new github issue
- Provide a description of the symptoms
- Provide instructions to reproduce the scenario
- Provide a description of your current setup (versions, configuration, environment)

## Feature Request

Please follow these basic guidelines when filling a feature request:

- Ensure that your request is within the [scope](#scope-of-the-project) of the project.
- Create a new github issue
- Provide a description of your use case (how would you use the new feature)

## Pull Requests

All PRs are welcome, but please take into account these requirements for your contribution to be considered by the maintainers:

- Ensure that your change is within the [scope](#scope-of-the-project) of the project.
- Fork the project, make your changes and create a PR in the upstream repository.
- Keep the scope of the PR as small as possible.
- Don't include changes unrelated to the scope of the PR, no matter how small or trivial they might seem.
- After you submit your pull request, verify that all status checks are passing.
