#!/usr/bin/env bash
# Offloads heavy Rust Cargo builds, Clippy lints, and integration test suites
# from local workstation (`darter`) to dedicated 8-core compute host `solidstate` (192.168.11.25).
set -euo pipefail

REMOTE_HOST="${ELLA_REMOTE_BUILD_HOST:-robert@192.168.11.25}"
REMOTE_DIR="${ELLA_REMOTE_BUILD_DIR:-~/builds/ella-world}"
REMOTE_JOBS="${ELLA_REMOTE_BUILD_JOBS:-8}"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [ "$#" -eq 0 ]; then
  set -- test
fi

echo "==> Syncing sources to compute node ${REMOTE_HOST}:${REMOTE_DIR}..."
rsync -az \
  --exclude 'target/' \
  --exclude '.git/' \
  --exclude 'scratch/' \
  --exclude 'assets/blender/' \
  --exclude 'data/' \
  "$ROOT_DIR/" "${REMOTE_HOST}:${REMOTE_DIR}/"

echo "==> Running remote cargo $* on ${REMOTE_HOST} (CARGO_BUILD_JOBS=${REMOTE_JOBS})..."
ssh -o BatchMode=yes "$REMOTE_HOST" "export PATH=\"\$HOME/.cargo/bin:\$PATH\" CARGO_BUILD_JOBS=${REMOTE_JOBS} && cd ${REMOTE_DIR} && cargo $*"

# If a release binary was built on solidstate, pull it back into local target/release/ ready for deploy
case " $* " in
  *" --release "*)
    if ssh -o BatchMode=yes "$REMOTE_HOST" "test -f ${REMOTE_DIR}/target/release/ella-world-server"; then
      mkdir -p "$ROOT_DIR/target/release"
      echo "==> Fetching compiled release binary ella-world-server from ${REMOTE_HOST}..."
      scp -q "${REMOTE_HOST}:${REMOTE_DIR}/target/release/ella-world-server" "$ROOT_DIR/target/release/ella-world-server"
      chmod 755 "$ROOT_DIR/target/release/ella-world-server"
    fi
    ;;
esac
