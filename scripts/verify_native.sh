#!/usr/bin/env bash
# Native host verification gate for ELLA World (zero cloud CI / zero GitHub Actions)
# Automatically offloads heavy Rust fmt + Clippy + integration tests to `water` (192.168.11.7)
# or `solidstate` (192.168.11.25) so local workstation `darter` stays quiet.
set -euo pipefail

if [ -d "$HOME/.cargo/bin" ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> [3/4] Checking browser ES module & CLI script syntax & zero-SSE invariant..."
node --check public/js/voice.js
node --check public/js/ws_spine.js
node --check public/js/device_sig.js
node --check public/js/qr_svg.js
node --check public/js/app.js
node --check public/js/world3d.js
node --check scripts/ella_admin_user_cli.mjs
node --check scripts/rotate_prod_hmac_secret.mjs
node --check scripts/verify_prod_http_and_auth.mjs
if grep -rnE "EventSource|/api/ella-world/stream|/api/ella-world/sync" public/; then
  echo "ERROR: Legacy SSE or HTTP /sync polling reference detected in public/!" >&2
  exit 1
fi

REMOTE_DIR="${ELLA_REMOTE_BUILD_DIR:-~/builds/ella-world}"
SELECTED_HOST=""
if [ "${ELLA_FORCE_LOCAL_VERIFY:-0}" != "1" ]; then
  if [ -n "${ELLA_REMOTE_BUILD_HOST:-}" ]; then
    if ssh -o BatchMode=yes -o ConnectTimeout=2 "$ELLA_REMOTE_BUILD_HOST" 'test -x "$HOME/.cargo/bin/cargo"' 2>/dev/null; then
      SELECTED_HOST="$ELLA_REMOTE_BUILD_HOST"
    fi
  else
    for candidate in "robert@192.168.11.7" "robert@192.168.11.25"; do
      if ssh -o BatchMode=yes -o ConnectTimeout=2 "$candidate" 'test -x "$HOME/.cargo/bin/cargo"' 2>/dev/null; then
        SELECTED_HOST="$candidate"
        break
      fi
    done
  fi
fi

if [ -n "$SELECTED_HOST" ]; then
  echo "==> Offloading Rust fmt + Clippy + test suite to compute host ${SELECTED_HOST}..."
  ssh -o BatchMode=yes "$SELECTED_HOST" "mkdir -p ${REMOTE_DIR}"
  rsync -az \
    --exclude 'target/' \
    --exclude '.git/' \
    --exclude 'scratch/' \
    --exclude 'assets/blender/' \
    --exclude 'godot_client/assets/' \
    --exclude 'godot_client/build/' \
    --exclude 'godot_client/.godot/' \
    --exclude 'godot_client/rust_opus/target/' \
    --exclude 'public/download/*.apk' \
    --exclude 'public/download/*.aab' \
    --exclude 'data/' \
    "$ROOT_DIR/" "${SELECTED_HOST}:${REMOTE_DIR}/"

  ssh -o BatchMode=yes "$SELECTED_HOST" "set -euo pipefail
    export PATH=\"\$HOME/.cargo/bin:\$PATH\"
    export CARGO_BUILD_JOBS=\"\${ELLA_REMOTE_BUILD_JOBS:-8}\"
    cd ${REMOTE_DIR}
    echo '==> [1/4] Checking Rust formatting on remote compute host (cargo fmt --check)...'
    cargo fmt --check
    echo '==> [2/4] Running strict Clippy lints on remote compute host (cargo clippy --all-targets -- -D warnings)...'
    cargo clippy --all-targets -- -D warnings
    echo '==> [4/4] Running unit + end-to-end UDP, WebRTC SFU & Multi-Playground integration tests...'
    cargo test"
else
  echo "==> [1/4] Checking Rust formatting (cargo fmt --check)..."
  cargo fmt --check

  echo "==> [2/4] Running strict Clippy lints (cargo clippy --all-targets -- -D warnings)..."
  cargo clippy --all-targets -- -D warnings

  echo "==> [4/4] Running unit + end-to-end UDP & WebRTC SFU integration tests..."
  cargo test
fi

echo "✅ All native ELLA World verification gates passed!"
