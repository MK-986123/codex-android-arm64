#!/usr/bin/env bash
# Bootstrap script for building and installing codex on Android/Termux (ARM64).
#
# Run this script inside Termux on an Android device:
#   bash scripts/termux-bootstrap.sh
#
# STORAGE ACCESS (optional — only needed to work with files on /sdcard):
#   1. Run:  termux-setup-storage
#      (If ~/storage already exists, confirm wiping — safe, only rebuilds symlinks.)
#   2. Grant the Storage permission when the Android dialog appears.
#   3. Verify:  ls ~/storage/shared
#
#   Symlinks created under ~/storage/:
#     shared/     — root of shared storage (/sdcard)
#     downloads/  — system Downloads folder
#     dcim/       — DCIM (camera photos/videos)
#     pictures/   — Pictures
#     music/      — Music
#     movies/     — Movies
#     external-1/ — external SD card (if available)
#
#   Android 11+ workaround if you still get "Permission denied" after granting:
#     Settings → Apps → Termux → Permissions → Storage
#     → Revoke → Grant again
#   (Known Android bug, not a Termux issue.)
#
# What this script does:
#   1. Installs required system packages via pkg
#   2. Builds the Rust codex binary
#   3. Installs codex to $PREFIX/bin
#   4. Installs Node.js dependencies for codex-cli
#   5. Sets up the ripgrep binary for codex-cli search
#   6. Writes persistent environment variables to ~/.bashrc

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX="${PREFIX:-/data/data/com.termux/files/usr}"
TMPDIR="${TMPDIR:-$PREFIX/tmp}"
INSTALL_BIN="$PREFIX/bin/codex"

log() { printf '\033[1;32m[termux-bootstrap]\033[0m %s\n' "$*"; }
die() { printf '\033[1;31m[termux-bootstrap] ERROR:\033[0m %s\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# 0. Device probe — print environment info for diagnostics
# ---------------------------------------------------------------------------
log "=== device probe ==="
uname -a || true
getprop ro.product.cpu.abi 2>/dev/null || true
getprop ro.build.version.sdk 2>/dev/null || true
getprop ro.product.model 2>/dev/null || true
echo "HOME=$HOME"
echo "PREFIX=${PREFIX:-}"
echo "TERMUX_VERSION=${TERMUX_VERSION:-}"
echo "TMPDIR=${TMPDIR:-}"
for x in sh bash git node npm rustc cargo clang; do
    printf '%-10s ' "$x"; command -v "$x" 2>/dev/null || echo "(not found)"
done
echo "=== end probe ==="

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
    pkg-config \
    git \
    make \
    binutils \
    ripgrep \
    zlib \
    libsqlite \
    patchelf \
    strace

# pnpm is used by the TypeScript layer
if ! command -v pnpm &>/dev/null; then
    log "Installing pnpm..."
    npm install -g pnpm
fi

# ---------------------------------------------------------------------------
# 2. Environment for OpenSSL (required by some Rust crates)
# ---------------------------------------------------------------------------
# In Termux, `openssl` installs libs and headers under $PREFIX (no -dev split).
export OPENSSL_DIR="$PREFIX"
export OPENSSL_LIB_DIR="$PREFIX/lib"
export OPENSSL_INCLUDE_DIR="$PREFIX/include"
export OPENSSL_NO_VENDOR=1
export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig"

# Termux sets TMPDIR to $PREFIX/tmp (not /tmp); ensure it's exported.
export TMPDIR="${TMPDIR:-$PREFIX/tmp}"

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
# The bundled codex-cli/bin/rg is a dotslash launcher with no Android entry.
# Replace it with a symlink to the system rg (installed above via pkg).
RG_TARGET="$REPO_ROOT/codex-cli/bin/rg"
if command -v rg &>/dev/null; then
    log "Linking system ripgrep into codex-cli/bin/rg..."
    ln -sf "$(command -v rg)" "$RG_TARGET"
else
    die "ripgrep not found after install — check pkg output above"
fi

# ---------------------------------------------------------------------------
# 7. Write persistent environment variables to ~/.bashrc
# ---------------------------------------------------------------------------
BASHRC="${HOME}/.bashrc"
MARKER="# codex-termux-env"
if ! grep -q "$MARKER" "$BASHRC" 2>/dev/null; then
    log "Writing persistent env vars to $BASHRC..."
    cat >> "$BASHRC" <<'EOF'

# codex-termux-env — added by scripts/termux-bootstrap.sh
export OPENSSL_DIR="$PREFIX"
export OPENSSL_LIB_DIR="$PREFIX/lib"
export OPENSSL_INCLUDE_DIR="$PREFIX/include"
export OPENSSL_NO_VENDOR=1
export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig"
# codex-termux-env-end
EOF
    log "Env vars written. Run: source ~/.bashrc"
else
    log "Persistent env vars already present in $BASHRC — skipping."
fi

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
log ""
log "✓ codex installed to $INSTALL_BIN"
log ""
log "Make sure OPENAI_API_KEY is set before running codex:"
log "  export OPENAI_API_KEY=sk-..."
log ""
log "For shared storage access (/sdcard → ~/storage/shared):"
log "  termux-setup-storage   # run once, grant Storage permission"
log "  Android 11+ fix if still denied: Settings → Apps → Termux → Permissions → Storage → Revoke → Grant"
