#!/bin/sh
# Runs inside the LocalStack container once it is ready.
# Creates the bucket that daemon.toml writes to.
awslocal s3 mb s3://my-bucket
