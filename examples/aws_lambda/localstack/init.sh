#!/bin/sh
# Runs inside the LocalStack container once it is ready.
# Packages handler.py and deploys it as the function daemon.toml invokes.
cd /etc/localstack/init/ready.d
python3 -c "import zipfile; zipfile.ZipFile('/tmp/fn.zip','w').write('handler.py')"
awslocal lambda create-function \
  --function-name my-lambda \
  --runtime python3.12 \
  --handler handler.handler \
  --role arn:aws:iam::000000000000:role/lambda-role \
  --zip-file fileb:///tmp/fn.zip
