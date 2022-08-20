# Advanced Features

This section provides detailed information on the some of the advanced features available in Oura:

- [Stateful Cursor](./stateful_cursor.md): provides a mechanism to persist the "position" of the processing pipeline to make it resilient to restarts.
- [Rollback Buffer](./rollback_buffer.md): provides a way to mitigate the impact of chain rollbacks in downstream stages.
- [Pipeline Metrics](./pipeline_metrics.md): allows operators to track the progress and performance of long-running Oura sessions.
- [Mapper Options](./mapper_options.md): A set of "expensive" event mapping procedures that require an explicit opt-in to be activated.
- [Intersect Options](./intersect_options.md): Advanced options for instructing Oura from which point in the chain to start reading from.
- [Custom Network](./custom_network.md): Instructions on how to configure Oura for connecting to a custom network.
- [Retry Policy](./retry_policy.md): Instructions on how to configure retry policies for different operations