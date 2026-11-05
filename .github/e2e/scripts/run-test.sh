#!/usr/bin/env bash
#
# Run a single e2e leg: resolve its config, run the release image, and let the
# process exit code decide pass/fail.
#
# oura self-terminates via the WorkStats filter's finalization policy (exit 0);
# the Assert sink panics on a bad block (non-zero); `timeout` guards a hang (124).
#
# Expects in the environment (set by the workflow):
#   TEST_NAME, GITHUB_RUN_NUMBER, RUNNER_TEMP, TARGET_IMAGE

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

# forward any AWS_* env the workflow exported (the OIDC step, on aws legs)
for var in $(compgen -e | grep '^AWS_' || true); do
  docker_args+=(-e "$var")
done

timeout 1800 docker run "${docker_args[@]}" "$TARGET_IMAGE" daemon
