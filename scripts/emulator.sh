#!/usr/bin/env bash
# Build (once) and start the AWS Emulator in Docker, then wait until it is healthy.
# Usage: scripts/emulator.sh [start|stop|env]
set -euo pipefail

PORT="${PORT:-9297}"
NAME="labcloud-emulator"
IMAGE="labcloud/aws-emulator"
DIR="$HOME/.aws/emulator"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

case "$(uname -m)" in
  arm64|aarch64) ARCH=aarch64 ;;
  *) ARCH=x86_64 ;;
esac

start() {
  if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    docker build --build-arg ARCH="$ARCH" -t "$IMAGE" "$ROOT/emulator"
  fi
  if [ -z "$(docker ps -q -f name="^${NAME}$")" ]; then
    mkdir -p "$DIR"
    # 127.0.0.1 only: never expose the emulator to the network.
    docker run -d --rm --name "$NAME" --memory=2g --memory-swap=2g \
      -p "127.0.0.1:$PORT:$PORT" -e HOME="$HOME" -v "$DIR:$DIR" \
      "$IMAGE" -p "$PORT" --telemetry off >/dev/null
  fi
  for _ in $(seq 60); do
    if curl -sf --cacert "$DIR/$PORT/ca.pem" "https://127.0.0.1:$PORT/__admin/health" >/dev/null 2>&1; then
      echo "AWS Emulator ready on https://127.0.0.1:$PORT"
      return 0
    fi
    sleep 1
  done
  echo "AWS Emulator did not become healthy; see: docker logs $NAME" >&2
  return 1
}

# Prints the exports that point the AWS CLI / SDKs at the emulator: eval "$(scripts/emulator.sh env)"
env_vars() {
  cat <<VARS
unset AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN AWS_PROFILE
export AWS_ENDPOINT_URL=https://127.0.0.1:$PORT
export AWS_SHARED_CREDENTIALS_FILE=$DIR/$PORT/emulator_credentials
export AWS_CONFIG_FILE=$DIR/$PORT/config
export AWS_CA_BUNDLE=$DIR/$PORT/ca.pem
export SSL_CERT_FILE=$DIR/$PORT/ca.pem
export AWS_EC2_METADATA_DISABLED=true
export AWS_DISABLE_HOST_PREFIX_INJECTION=true
export AWS_REGION=us-east-1
export AWS_DEFAULT_REGION=us-east-1
VARS
}

case "${1:-start}" in
  start) start ;;
  stop) docker stop "$NAME" >/dev/null && echo "stopped" ;;
  env) env_vars ;;
  *) echo "usage: $0 [start|stop|env]" >&2; exit 2 ;;
esac
