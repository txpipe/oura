#!/usr/bin/env bash
#
# Best-effort teardown of the dmtrcli tunnel started by setup-dmtr-socket.sh.
#
# Expects in the environment (set by the workflow): DMTR_TUNNEL_PID (optional)

kill "${DMTR_TUNNEL_PID:-}" 2>/dev/null || true
