#!/bin/sh
set -e

REPO="Alfredvc/bugsink-cli"
INSTALL_DIR="${BUGSINK_INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)  OS="linux" ;;
  darwin) OS="darwin" ;;
  *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

ARTIFACT="bugsink-${OS}-${ARCH}"

# Get latest release tag
if [ -z "$BUGSINK_VERSION" ]; then
  BUGSINK_VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  if [ -z "$BUGSINK_VERSION" ]; then
    echo "Error: could not determine latest release version" >&2
    exit 1
  fi
fi

URL="https://github.com/${REPO}/releases/download/${BUGSINK_VERSION}/${ARTIFACT}.tar.gz"

echo "Downloading bugsink ${BUGSINK_VERSION} for ${OS}/${ARCH}..."
curl -fsSL "$URL" | tar xz -C "$INSTALL_DIR"
echo "Installed bugsink to ${INSTALL_DIR}/bugsink"
echo "Run 'bugsink --help' to get started."
