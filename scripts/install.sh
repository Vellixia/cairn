#!/bin/sh
# Cairn installer (Linux/macOS).
#
#   curl -fsSL https://cairn.sh/install.sh | sh
#   curl -fsSL https://cairn.sh/install.sh | sh -s -- pair CAIRN-XXXX   # install + pair a device
#
# Honors: CAIRN_REPO, CAIRN_INSTALL_DIR.
set -eu

REPO="${CAIRN_REPO:-cairn-dev/cairn}"
BIN="cairn"
INSTALL_DIR="${CAIRN_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf '\033[36m›\033[0m %s\n' "$1"; }
err() { printf '\033[31m✗ %s\033[0m\n' "$1" >&2; exit 1; }

detect_target() {
  os="$(uname -s)"; arch="$(uname -m)"
  case "$os" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *) err "unsupported OS: $os (use install.ps1 on Windows)" ;;
  esac
  case "$arch" in
    x86_64|amd64)  arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *) err "unsupported arch: $arch" ;;
  esac
  printf '%s-%s' "$arch" "$os"
}

install_binary() {
  target="$(detect_target)"
  say "Installing cairn ($target) -> $INSTALL_DIR"
  mkdir -p "$INSTALL_DIR"
  url="https://github.com/$REPO/releases/latest/download/cairn-$target.tar.gz"
  tmp="$(mktemp -d)"
  if curl -fsSL "$url" -o "$tmp/cairn.tar.gz" 2>/dev/null; then
    tar -xzf "$tmp/cairn.tar.gz" -C "$tmp"
    mv "$tmp/$BIN" "$INSTALL_DIR/$BIN"
    chmod +x "$INSTALL_DIR/$BIN"
  elif command -v cargo >/dev/null 2>&1; then
    say "No prebuilt release found; building from source with cargo…"
    cargo install --git "https://github.com/$REPO" cairn-cli
  else
    err "no prebuilt binary available and cargo is not installed"
  fi
  rm -rf "$tmp"
}

install_binary
case ":$PATH:" in
  *":$INSTALL_DIR:"*) : ;;
  *) say "Add $INSTALL_DIR to your PATH to use \`cairn\` everywhere." ;;
esac

# Optional: `... | sh -s -- pair CODE` pairs the device, then wires up local agents.
if [ "${1:-}" = "pair" ] && [ -n "${2:-}" ]; then
  say "Pairing this device…"
  "$INSTALL_DIR/$BIN" pair "$2" || err "pairing failed"
  say "Configuring installed agents…"
  "$INSTALL_DIR/$BIN" install --all || true
fi

say "Done. Start the server with:  cairn serve"
