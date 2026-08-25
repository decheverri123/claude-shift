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

# Verify the tarball against published SHA256 checksums for this asset.
if [ -f "$tmp/SHA256SUMS" ]; then
  if grep -F "$asset" "$tmp/SHA256SUMS" > "$tmp/checksum.txt" 2>/dev/null; then
    if command -v sha256sum >/dev/null 2>&1; then
      (cd "$tmp" && sha256sum -c checksum.txt)
    elif command -v shasum >/dev/null 2>&1; then
      (cd "$tmp" && shasum -a 256 -c checksum.txt)
    else
      echo "warning: neither sha256sum nor shasum found; skipping checksum verification." >&2
    fi || {
      echo "error: tarball hash mismatch; refusing to install." >&2
      exit 1
    }
  fi
fi

mkdir -p "$tmp/extracted" "$INSTALL_DIR"
tar xzf "$tmp/$asset" -C "$tmp/extracted"

if [ -f "$tmp/extracted/$BIN.exe" ]; then
  mv -f "$tmp/extracted/$BIN.exe" "$INSTALL_DIR/$BIN.exe"
elif [ -f "$tmp/extracted/$BIN" ]; then
  mv -f "$tmp/extracted/$BIN" "$INSTALL_DIR/$BIN"
fi
chmod +x "$INSTALL_DIR/$BIN" 2>/dev/null || true

echo "Installed $BIN to $INSTALL_DIR"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add it to your PATH: export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac
