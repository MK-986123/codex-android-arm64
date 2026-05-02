#!/data/data/com.termux/files/usr/bin/bash
# Bootstrap script for building and installing codex on Android/Termux (ARM64).
#
# Run this script inside Termux on an Android device:
#   bash scripts/termux-bootstrap.sh
#
# What it does:
#   1. Installs required system packages via pkg
#   2. Installs Node.js packages for codex-cli
#   3. Builds the Rust TUI binary (codex-rs)
#   4. Installs the codex binary to $PREFIX/bin

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX="${PREFIX:-/data/data/com.termux/files/usr}"
TMPDIR="${TMPDIR:-$PREFIX/tmp}"
INSTALL_BIN="$PREFIX/bin/codex"

log() { printf '\033[1;32m[termux-bootstrap]\033[0m %s\n' "$*"; }
die() { printf '\033[1;31m[termux-bootstrap] ERROR:\033[0m %s\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# 1. System packages
# ---------------------------------------------------------------------------
log "Updating package lists..."
pkg update -y

log "Installing build dependencies..."
pkg install -y \
    rust \
    nodejs-lts \
    openssl \
    openssl-dev \
    pkg-config \
    git \
    make \
    binutils

# pnpm is used by the TypeScript layer
if ! command -v pnpm &>/dev/null; then
    log "Installing pnpm..."
    npm install -g pnpm
fi

# ---------------------------------------------------------------------------
# 2. Environment for OpenSSL (required by some Rust crates)
# ---------------------------------------------------------------------------
export OPENSSL_DIR="$PREFIX"
export OPENSSL_LIB_DIR="$PREFIX/lib"
export OPENSSL_INCLUDE_DIR="$PREFIX/include"
export OPENSSL_NO_VENDOR=1

# Termux sets TMPDIR; ensure it's exported for child processes
export TMPDIR

# ---------------------------------------------------------------------------
# 3. Build codex-rs TUI
# ---------------------------------------------------------------------------
log "Building codex Rust binary..."
cd "$REPO_ROOT/codex-rs"
# codex-cli crate (codex-rs/cli/) produces the `codex` binary
cargo build --release -p codex-cli

CODEX_BIN="$REPO_ROOT/codex-rs/target/release/codex"
[ -f "$CODEX_BIN" ] || die "Build succeeded but binary not found at $CODEX_BIN"

# ---------------------------------------------------------------------------
# 4. Install Node.js dependencies for codex-cli
# ---------------------------------------------------------------------------
log "Installing Node.js dependencies..."
cd "$REPO_ROOT"
pnpm install --frozen-lockfile 2>/dev/null || pnpm install

# ---------------------------------------------------------------------------
# 5. Install the codex binary
# ---------------------------------------------------------------------------
log "Installing codex to $INSTALL_BIN..."
install -m 755 "$CODEX_BIN" "$INSTALL_BIN"

# ---------------------------------------------------------------------------
# 6. Install an ARM64 ripgrep binary (used by codex-cli for search)
# ---------------------------------------------------------------------------
RG_TARGET="$REPO_ROOT/codex-cli/bin/rg"
if [ ! -x "$RG_TARGET" ] || file "$RG_TARGET" 2>/dev/null | grep -qv "aarch64\|ARM64"; then
    if command -v rg &>/dev/null; then
        log "Linking system ripgrep into codex-cli/bin/rg..."
        ln -sf "$(command -v rg)" "$RG_TARGET"
    else
        log "Installing ripgrep via pkg..."
        pkg install -y ripgrep
        ln -sf "$(command -v rg)" "$RG_TARGET"
    fi
fi

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
log "Done! Run 'codex' to start."
log "Make sure OPENAI_API_KEY is set in your environment."
log ""
log "Tip: Add to ~/.bashrc or ~/.zshrc:"
log "  export OPENSSL_DIR=\"\$PREFIX\""
log "  export OPENSSL_LIB_DIR=\"\$PREFIX/lib\""
log "  export OPENSSL_INCLUDE_DIR=\"\$PREFIX/include\""
