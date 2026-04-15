#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGE="alfalfa-standalone-desktop"
BUILD_DIR="$SCRIPT_DIR/build"
DOCKER_ARGS=()

for arg in "$@"; do
  case "$arg" in
    --no-cache) DOCKER_ARGS+=(--no-cache) ;;
    *) echo "Unknown option: $arg"; echo "Usage: $0 [--no-cache]"; exit 1 ;;
  esac
done

echo "==> Building Docker image..."
docker build "${DOCKER_ARGS[@]}" -t "$IMAGE" "$SCRIPT_DIR"

echo "==> Extracting artifacts..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

docker run --rm --user "$(id -u):$(id -g)" \
  -v "$BUILD_DIR:/output" \
  "$IMAGE" \
  sh -c '
    cp /app/packages/desktop/src-tauri/target/release/AlfAlfa /output/
    cp /app/packages/desktop/src-tauri/target/release/bundle/deb/*.deb /output/ 2>/dev/null || true
    cp /app/packages/desktop/src-tauri/target/release/bundle/rpm/*.rpm /output/ 2>/dev/null || true
  '

echo ""
echo "==> Build complete. Artifacts:"
ls -lh "$BUILD_DIR"
