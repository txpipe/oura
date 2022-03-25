# AWS SQS

A sink that sends each event as message to an AWS SQS queue. Each event is json-encoded and sent to a configurable SQS queue using AWS API endpoints.

The sink will process each incoming event in sequence and submit the corresponding `SendMessage` request to the SQS API. Once the queue acknowledges reception of the message, the sink will advance and continue with the following event.

The sink support both FIFO and Standard queues. The sink configuration will determine which logic to apply. In case of FIFO, the group id is determined by an explicit configuration value and the message id is defined by the fingerprint value of the event (Fingerprint filter needs to be enabled). 

The sink uses AWS SDK's built-in retry logic which can be configured at the sink level. Authentication against AWS is built-in in the SDK library and follows the common chain of providers (env vars, ~/.aws, etc). 

## Configuration

```toml
[sink]
type = "AwsSqs"
region = "us-west-2"
queue_url = "https://sqs.us-west-2.amazonaws.com/xxxxxx/my-queue"
fifo = true
group_id = "my_group"
max_retries = 5
```

### Section: `sink`

- `type`: the literal value `AwsSqs`.

- `region`: The AWS region where the queue is located.
- `queue_url`: The SQS queue URL provided by AWS (not to be confused with the ARN).
- `fifo`: A flag to determine if the queue is of type FIFO.
- `group_id`: A fixed group id to be used when sending messages to a FIFO queue.
- `max_retries`: The max number of send retries before exiting the pipeline with an error.

## AWS Credentials

The sink needs valid AWS credentials to interact with the cloud service. The mayority of the SDKs and libraries that interact with AWS follow the same approach to access these credentials from a chain of possible providers:

- Credentials stored as the environment variables AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY.
- A Web Identity Token credentials from the environment or container (including EKS)
   ECS credentials (IAM roles for tasks)
- As entries in the credentials file in the .aws directory in your home directory (~/.aws/
- From the EC2 Instance Metadata Service (IAM Roles attached to an instance)

Oura, by mean of the Rust AWS SDK lib, will honor the above chain of providers. Use any of the above that fits your particular scenario. Please refer to AWS' documentation for more detail.

## FIFO vs Standard Queues

Oura processes messages maintaining the sequence of the blocks and respecting the order of the transactions within the block. An AWS SQS FIFO queue provides the consumer with a guarantee that the order in which messages are consumed matches the order in which they were sent.

Connecting Oura with a FIFO queue would provide the consumer with the guarantee that the events received follow the same order as they appeared in the blockchain. This might be useful, for example, in scenarios where the processing of an event requires a reference of a previous state of the chain.

Please note that rollback events might happen upstream, at the blockchain level, which need to be handled by the consumer to unwind any side-effects of the processing of the newly orphaned blocks. This problem can be mitigated by using Oura's [rollback buffer](../advanced/rollback_buffer.md) feature.

If each event can be processed in isolation, if the process is idempotent or if the order doesn't affect the outcome, the recommended approach is to use a Standard queue which provides "at least once" processing guarantees, relaxing the constraints and improving the overall performance.

## Payload Size Limitation

AWS SQS service has a 256kb payload size limit. This is more than enough for individual events, but it might be too little for pipelines where the `include_cbor_hex` option is enabled. If your goal of your pipeline is to acccess the raw CBOR content, we recommend taking a look at the [AWS S3 Sink](./aws_s3.md) that provides a direct way for storing CBOR block in an S3 bucket.
