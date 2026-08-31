#!/bin/sh
# Install lockguard from GitHub Releases
# Usage: curl -fsSL https://github.com/alinaqi2000/lockguard/releases/latest/download/install.sh | sh

set -e

ARCH=$(uname -m)
OS=$(uname -s)

case "$OS" in
    Linux) TARGET_OS="unknown-linux-gnu" ;;
    Darwin) TARGET_OS="apple-darwin" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64) TARGET_ARCH="x86_64" ;;
    aarch64|arm64) TARGET_ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"
REPO="alinaqi2000/lockguard"

echo "Fetching latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
echo "Latest version: ${LATEST}"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

URL="https://github.com/${REPO}/releases/download/${LATEST}/lockguard-${TARGET}.tar.gz"
echo "Downloading ${URL}..."
curl -fsSL "$URL" | tar xz -C "$TMPDIR"

echo "Installing to /usr/local/bin..."
sudo install -m 755 "$TMPDIR/lockguard" /usr/local/bin/lockguard

echo "Done. Run: lockguard --help"
