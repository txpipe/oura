# Advanced Features

This section provides detailed information on the some of the advanced features available in Oura:

- [Stateful Cursor](./stateful_cursor.md): The _cursor_ feature provides a mechanism to persist the "position" of the processing pipeline to make it resilient to restarts.
- [Rollback Buffer](./rollback_buffer.md): The "rollback buffer" feature provides a way to mitigate the impact of chain rollbacks in downstream stages of the data-processing pipeline.
- [Pipeline Metrics](./pipeline_metrics.md): The _metrics_ features allows operators to track the progress and performance of long-running Oura sessions.
- [Mapper Options](./mapper_options.md): A set of "expensive" event mapping procedures that require an explicit opt-in to be activated.
- [Intersect Options](./intersect_options.md): Advanced options for instructing Oura from which point in the chain to start reading from.
- [Well-known Chain Info](./wellknown_chain_info.md):