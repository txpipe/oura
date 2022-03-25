# AWS S3

A sink that saves the CBOR content of the blocks as S3 object.

The sink will process each incoming event in sequence and select only the events of type `Block`. The CBOR content of the block will be extracted and saves as an S3 object in a configurable bucket in either hex or binary encoding. 

A configurable option allows the user to decide how to name the object using values from the block header (such as epoch, slot, hash, etc). The properties of the block will be saved as metadata of the S3 Object for later identification (eg: block number, hash, slot, etc).

The sink uses AWS SDK's built-in retry logic which can be configured at the sink level. Authentication against AWS is built-in in the SDK library and follows the common chain of providers (env vars, ~/.aws, etc). 

## Configuration

```toml
[sink]
type = "AwsS3"
region = "us-west-2"
bucket = "my-bucket"
prefix = "mainnet/"
naming = "SlotHash"
content = "Cbor"
max_retries = 5
```

### Section: `sink`

- `type`: the literal value `AwsS3`.
- `function_name`: The ARN of the function we wish to invoke.
- `region`: The AWS region where the bucket is located.
- `bucket`: The name of the bucket to store the blocks.
- `prefix`: A prefix to prepend on each object's key.
- `naming`: One of the available naming conventions (see section below)
- `content`: Either `Cbor` for binary encoding or `CborHex` for plain text hex representation of the CBOR
- `max_retries`: The max number of send retries before exiting the pipeline with an error.

IMPORTANT: For this sink to work correctly, the `include_block_cbor` option should be enabled in the source sink configuration (see [mapper options](../advanced/mapper_options.md)).

## Naming Convention

S3 Buckets allow the user to query by object prefix. It's important to use a naming convention that is compatible with the types of filters that the consumer intends to use. This sink provides the following options:


- `Hash`: formats the key using `"{hash}"`
- `SlotHash`: formats the key using `"{slot}.{hash}"`
- `BlockHash`: formats the key using `"{block_num}.{hash}"`
- `EpochHash`: formats the key using `"{epoch}.{hash}"`
- `EpochSlotHash`: formats the key using `"{epoch}.{slot}.{hash}"`
- `EpochBlockHash`: formats the key using `"{epoch}.{block_num}.{hash}"`

## Content Encoding

The sink provides two options for encoding the content of the object:

- `Cbor`: the S3 object will contain the raw, unmodified CBOR value in binary format. The content type of the object in this case will be "application/cbor". 
- `CborHex`: the S3 object will contain the CBOR payload of the block encoded as a hex string. The content type of the object in this case will be "text/plain". 


## Metadata

The sink uses the data from the block event to populate metadata fields of the S3 object for easier identification of the block once persisted:

- `era`
- `issuer_vkey`
- `tx_count`
- `slot`
- `hash`
- `number`
- `previous_hash`

Please note that S3 doesn't allow filtering by metadata. For efficient filter, the only option available is to use the prefix of the key.

## AWS Credentials

The sink needs valid AWS credentials to interact with the cloud service. The majority of the SDKs and libraries that interact with AWS follow the same approach to access these credentials from a chain of possible providers:

- Credentials stored as the environment variables AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY.
- A Web Identity Token credentials from the environment or container (including EKS)
   ECS credentials (IAM roles for tasks)
- As entries in the credentials file in the .aws directory in your home directory (~/.aws/
- From the EC2 Instance Metadata Service (IAM Roles attached to an instance)

Oura, by mean of the Rust AWS SDK lib, will honor the above chain of providers. Use any of the above that fits your particular scenario. Please refer to AWS' documentation for more detail.
