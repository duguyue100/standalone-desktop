#!/usr/bin/env bash
set -euo pipefail

REPO="duguyue100/standalone-desktop"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="AlfAlfa"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  PLATFORM="linux" ;;
  Darwin) PLATFORM="macos" ;;
  *)      echo "Error: Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_LABEL="x86_64" ;;
  arm64|aarch64) ARCH_LABEL="aarch64" ;;
  *)             echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
esac

# macOS only supports aarch64, Linux only supports x86_64 for now
if [ "$PLATFORM" = "macos" ] && [ "$ARCH_LABEL" = "x86_64" ]; then
  echo "Error: macOS x86_64 builds are not available. Only Apple Silicon (aarch64) is supported."
  exit 1
fi
if [ "$PLATFORM" = "linux" ] && [ "$ARCH_LABEL" = "aarch64" ]; then
  echo "Error: Linux aarch64 builds are not available. Only x86_64 is supported."
  exit 1
fi

ASSET_NAME="${BINARY_NAME}-${PLATFORM}-${ARCH_LABEL}"

echo "Detecting platform... $PLATFORM $ARCH_LABEL"

# Fetch the latest release download URL
echo "Fetching latest release..."
DOWNLOAD_URL=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep -o "\"browser_download_url\": *\"[^\"]*${ASSET_NAME}\"" \
  | head -1 \
  | cut -d'"' -f4)

if [ -z "$DOWNLOAD_URL" ]; then
  echo "Error: Could not find asset '${ASSET_NAME}' in the latest release."
  echo "Check https://github.com/${REPO}/releases for available downloads."
  exit 1
fi

# Download
echo "Downloading ${ASSET_NAME}..."
TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT
curl -fSL --progress-bar "$DOWNLOAD_URL" -o "$TMPFILE"

# Install
mkdir -p "$INSTALL_DIR"
mv "$TMPFILE" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo ""
echo "AlfAlfa installed to ${INSTALL_DIR}/${BINARY_NAME}"

# Check if install dir is in PATH
case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "WARNING: ${INSTALL_DIR} is not in your PATH."
    echo ""
    SHELL_NAME=$(basename "${SHELL:-/bin/bash}")
    case "$SHELL_NAME" in
      fish)
        echo "Add it by running:"
        echo "  fish_add_path ${INSTALL_DIR}"
        ;;
      zsh)
        echo "Add it to your ~/.zshrc:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        ;;
      *)
        echo "Add it to your ~/.bashrc (or ~/.profile):"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        ;;
    esac
    echo ""
    echo "Then restart your shell or run the command above."
    ;;
esac
