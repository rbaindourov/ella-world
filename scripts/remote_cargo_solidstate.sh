#!/usr/bin/env bash
# Offloads heavy Rust Cargo builds, Clippy lints, and integration test suites
# from local workstation (`darter`) to dedicated LAN compute hosts (`water` 192.168.11.7 or `solidstate` 192.168.11.25).
set -euo pipefail

select_remote_host() {
  if [ -n "${ELLA_REMOTE_BUILD_HOST:-}" ]; then
    echo "$ELLA_REMOTE_BUILD_HOST"
    return
  fi
  for candidate in "robert@192.168.11.7" "robert@192.168.11.25"; do
    if ssh -o BatchMode=yes -o ConnectTimeout=2 "$candidate" 'test -x "$HOME/.cargo/bin/cargo"' 2>/dev/null; then
      echo "$candidate"
      return
    fi
  done
  echo "ERROR: Neither water (192.168.11.7) nor solidstate (192.168.11.25) is reachable with cargo installed." >&2
  exit 1
}

REMOTE_HOST="$(select_remote_host)"
REMOTE_DIR="${ELLA_REMOTE_BUILD_DIR:-~/builds/ella-world}"
REMOTE_JOBS="${ELLA_REMOTE_BUILD_JOBS:-8}"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [ "$#" -eq 0 ]; then
  set -- test
fi

echo "==> Syncing sources to compute node ${REMOTE_HOST}:${REMOTE_DIR}..."
ssh -o BatchMode=yes "$REMOTE_HOST" "mkdir -p ${REMOTE_DIR}"
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
  "$ROOT_DIR/" "${REMOTE_HOST}:${REMOTE_DIR}/"

echo "==> Running remote cargo $* on ${REMOTE_HOST} (CARGO_BUILD_JOBS=${REMOTE_JOBS})..."
ssh -o BatchMode=yes "$REMOTE_HOST" "export PATH=\"\$HOME/.cargo/bin:\$PATH\" CARGO_BUILD_JOBS=${REMOTE_JOBS} && cd ${REMOTE_DIR} && cargo $*"

# If a release binary was built on the remote compute node, pull it back into local target/release/ ready for deploy
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
