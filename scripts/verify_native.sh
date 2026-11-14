#!/usr/bin/env bash
# Native host verification gate for ELLA World (zero cloud CI / zero GitHub Actions)
set -euo pipefail

if [ -d "$HOME/.cargo/bin" ]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> [1/4] Checking Rust formatting (cargo fmt --check)..."
cargo fmt --check

echo "==> [2/4] Running strict Clippy lints (cargo clippy --all-targets -- -D warnings)..."
cargo clippy --all-targets -- -D warnings

echo "==> [3/4] Checking browser ES module & CLI script syntax & zero-SSE invariant..."
node --check public/js/voice.js
node --check public/js/ws_spine.js
node --check public/js/app.js
node --check public/js/world3d.js
node --check scripts/ella_admin_user_cli.mjs
node --check scripts/rotate_prod_hmac_secret.mjs
node --check scripts/verify_prod_http_and_auth.mjs
if grep -rnE "EventSource|/api/ella-world/stream|/api/ella-world/sync" public/; then
  echo "ERROR: Legacy SSE or HTTP /sync polling reference detected in public/!" >&2
  exit 1
fi

echo "==> [4/4] Running unit + end-to-end UDP & WebRTC SFU integration tests..."
cargo test

echo "✅ All native ELLA World verification gates passed!"
