# AWS S3

A sink that saves the CBOR content of the blocks as S3 object.

The sink will process each input event in sequence and according to the configuration in daemon.toml it will save in S3 the block in CBOR format.

The location where the object will be saved can be configured by adding value in the prefix field, for example `mainnet/`

Authentication against AWS is built-in in the SDK library and follows the common chain of providers (env vars, ~/.aws, etc).

## Configuration

```toml
[sink]
type = "AwsS3"
region = "us-west-2"
bucket = "my-bucket"
prefix = "mainnet/"
```

### Section: `sink`

- `type`: the literal value `AwsS3`.
- `region`: The AWS region where the bucket is located.
- `bucket`: The name of the bucket to store the blocks.
- `prefix`: A prefix to prepend on each object's key.

IMPORTANT: Only the cbor block format is supported.

## Naming Convention

The name of the object and the slot number in which it was processed.

## Content Encoding

- `application/cbor`

## AWS Credentials

The sink needs valid AWS credentials to interact with the cloud service. The majority of the SDKs and libraries that interact with AWS follow the same approach to access these credentials from a chain of possible providers:

- Credentials stored as the environment variables AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY.
- A Web Identity Token credentials from the environment or container (including EKS)
- ECS credentials (IAM roles for tasks)
- As entries in the credentials file in the .aws directory in your home directory (~/.aws/)
- From the EC2 Instance Metadata Service (IAM Roles attached to an instance)

Oura, by mean of the Rust AWS SDK lib, will honor the above chain of providers. Use any of the above that fits your particular scenario. Please refer to AWS' documentation for more detail.
