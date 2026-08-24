#!/usr/bin/env bash
set -euo pipefail

REPO="decheverri123/claude-shift"
BIN="cshift"
INSTALL_DIR="${CSHIFT_INSTALL_DIR:-$HOME/.local/bin}"

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Darwin) os_tag="apple-darwin" ;;
  Linux) os_tag="unknown-linux-gnu" ;;
  MINGW*|MSYS*|CYGWIN*) os_tag="pc-windows-msvc" ;;
  *)
    echo "error: unsupported OS: $os" >&2
    exit 1
    ;;
esac

case "$arch" in
  x86_64|amd64) arch_tag="x86_64" ;;
  arm64|aarch64) arch_tag="aarch64" ;;
  *)
    echo "error: unsupported architecture: $arch" >&2
    exit 1
    ;;
esac

target="${arch_tag}-${os_tag}"
asset="cshift-${target}.tar.gz"
base="https://github.com/${REPO}/releases/latest/download"

echo "Downloading ${asset}..."
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl -fsSL "$base/${asset}" -o "$tmp/$asset"
curl -fsSL "$base/SHA256SUMS" -o "$tmp/SHA256SUMS"

# Verify the tarball against published SHA256 checksums.
(cd "$tmp" && sha256sum -c SHA256SUMS) || {
  echo "error: tarball hash mismatch; refusing to install." >&2
  exit 1
}

tar xzf "$tmp/$asset" -C "$tmp"

mkdir -p "$INSTALL_DIR"
mv "$tmp"/${BIN}* "$INSTALL_DIR/$BIN"
chmod +x "$INSTALL_DIR/$BIN" 2>/dev/null || true

echo "Installed $BIN to $INSTALL_DIR"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add it to your PATH: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
