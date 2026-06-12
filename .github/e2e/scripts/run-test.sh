#!/usr/bin/env bash
#
# Run a single e2e leg: resolve its config, run the release image, and let the
# process exit code decide pass/fail.
#
# oura self-terminates via the WorkStats filter's finalization policy (exit 0);
# the Assert sink panics on a bad block (non-zero); `timeout` guards a hang (124).
#
# Expects in the environment (set by the workflow):
#   TEST_NAME, KIND, GITHUB_RUN_NUMBER, RUNNER_TEMP, TARGET_IMAGE
# For aws legs, the AWS_* credentials must already be exported.

set -euo pipefail

# Resolve placeholders (e.g. the s3 prefix) into the final config.
envsubst < ".github/e2e/configs/${TEST_NAME}.toml" > "${RUNNER_TEMP}/daemon.toml"
echo "----- resolved daemon.toml -----"
cat "${RUNNER_TEMP}/daemon.toml"
echo "--------------------------------"

docker_args=(
  --rm
  -e RUST_LOG=warn
  -v "${RUNNER_TEMP}/daemon.toml:/etc/oura/daemon.toml:ro"
)

if [ "$KIND" = "aws" ]; then
  docker_args+=(
    -e AWS_ACCESS_KEY_ID
    -e AWS_SECRET_ACCESS_KEY
    -e AWS_SESSION_TOKEN
    -e AWS_REGION
    -e AWS_DEFAULT_REGION
  )
fi

if [ "$KIND" = "n2c" ]; then
  docker_args+=(-v "${RUNNER_TEMP}/sockets:/opt/cardano/cnode/sockets")
fi

timeout 1800 docker run "${docker_args[@]}" "$TARGET_IMAGE" daemon
