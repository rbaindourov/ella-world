#!/usr/bin/env bash
# Native host verification gate for ELLA World (zero cloud CI / zero GitHub Actions)
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
node --check public/js/app.js
node --check public/js/world3d.js
node --check scripts/ella_admin_user_cli.mjs
node --check scripts/rotate_prod_hmac_secret.mjs
node --check scripts/verify_prod_http_and_auth.mjs
if grep -rnE "EventSource|/api/ella-world/stream|/api/ella-world/sync" public/; then
  echo "ERROR: Legacy SSE or HTTP /sync polling reference detected in public/!" >&2
  exit 1
fi

REMOTE_HOST="${ELLA_REMOTE_BUILD_HOST:-robert@192.168.11.25}"
REMOTE_DIR="${ELLA_REMOTE_BUILD_DIR:-~/builds/ella-world}"

if [ "${ELLA_FORCE_LOCAL_VERIFY:-0}" != "1" ] && ssh -o BatchMode=yes -o ConnectTimeout=2 "$REMOTE_HOST" 'test -x "$HOME/.cargo/bin/cargo"' 2>/dev/null; then
  echo "==> Offloading Rust fmt + Clippy + test suite to compute host ${REMOTE_HOST} (solidstate)..."
  rsync -az \
    --exclude 'target/' \
    --exclude '.git/' \
    --exclude 'scratch/' \
    --exclude 'assets/blender/' \
    --exclude 'data/' \
    "$ROOT_DIR/" "${REMOTE_HOST}:${REMOTE_DIR}/"

  ssh -o BatchMode=yes "$REMOTE_HOST" "set -euo pipefail
    export PATH=\"\$HOME/.cargo/bin:\$PATH\"
    export CARGO_BUILD_JOBS=\"\${ELLA_REMOTE_BUILD_JOBS:-8}\"
    cd ${REMOTE_DIR}
    echo '==> [1/4] Checking Rust formatting on solidstate (cargo fmt --check)...'
    cargo fmt --check
    echo '==> [2/4] Running strict Clippy lints on solidstate (cargo clippy --all-targets -- -D warnings)...'
    cargo clippy --all-targets -- -D warnings
    echo '==> [4/4] Running unit + end-to-end UDP & WebRTC SFU integration tests on solidstate...'
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
