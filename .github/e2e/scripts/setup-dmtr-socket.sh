#!/usr/bin/env bash
#
# Expose a Demeter-managed Cardano node n2c socket on the runner host via
# dmtrcli, so the containerized oura N2C source can reach it.
#
# PLACEHOLDER: the exact dmtrcli auth/port-forward invocation must be confirmed
# against the real CLI. The n2c legs are continue-on-error until then.
#
# Expects in the environment (set by the workflow):
#   DMTR_API_KEY, RUNNER_TEMP, GITHUB_ENV

set -euo pipefail

curl -fsSL https://raw.githubusercontent.com/demeter-run/cli/main/install.sh | sh

mkdir -p "${RUNNER_TEMP}/sockets"

dmtrcli auth login --api-key "${DMTR_API_KEY}"

# Forward the managed node's n2c socket onto the runner host.
dmtrcli ports tunnel --unix-socket "${RUNNER_TEMP}/sockets/node0.socket" &
echo "DMTR_TUNNEL_PID=$!" >> "$GITHUB_ENV"

# Wait for the socket to appear before starting oura.
for _ in $(seq 1 30); do
  if [ -S "${RUNNER_TEMP}/sockets/node0.socket" ]; then break; fi
  sleep 2
done
