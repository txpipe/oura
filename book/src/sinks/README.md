# Sinks

Sinks are the "destination" of the events processed by _Oura_. They are the concrete link between the internal representation of the data records and the corresponding external service interested in the data. Typical sinks include: database engines, stream-processing engines, web API calls and FaaS solutions.

## Built-in Sinks

These are the existing sinks that are included as part the main _Oura_ codebase:

- [Terminal](terminal.md): a sink that outputs events into stdout with fancy coloring
- [Kakfa](kafka.md): a sink that sends each event into a Kafka topic
- [Elasticsearch](elastic.md): a sink that writes events into an Elasticsearch index or data stream.
- [Webhook](webhook.md): a sink that outputs each event as an HTTP call to a remote endpoint.
- [Logs](logs.md): a sink that saves events to the file system using JSONL text files.
- [AWS SQS](aws_sqs.md): a sink that sends each event as message to an AWS SQS queue.
- [AWS Lamda](aws_lambda.md): a sink that invokes an AWS Lambda function for each event.
- [AWS S3](aws_s3.md): a sink that saves the CBOR content of the blocks as an AWS S3 object.
- [GCP PubSub](gcp_pubsub.md): a sink that sends each event as a message to a google cloud PubSub topic.
- [GCP CloudFunction](gcp_cloudfunction.md): a sink that sends each event as JSON to a Cloud Function via HTTP.

New sinks are being developed, information will be added in this documentation to reflect the updated list. Contributions and feature request are welcome in our [Github Repo](https://github.com/txpipe/oura).