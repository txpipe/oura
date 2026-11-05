#!/bin/sh
# Runs inside the LocalStack container once it is ready.
# Creates the queue that daemon.toml writes to.
awslocal sqs create-queue --queue-name my-queue
