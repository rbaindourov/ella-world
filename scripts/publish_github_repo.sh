#!/usr/bin/env bash
# Publishes the standalone ELLA World repository to GitHub (public repo).
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

REMOTE_SSH="${1:-git@github.com:rbaindourov/ella-world.git}"

if ! git remote get-url origin >/dev/null 2>&1; then
  git remote add origin "$REMOTE_SSH"
fi

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  echo "==> Creating/verifying public GitHub repository via gh CLI..."
  gh repo view rbaindourov/ella-world >/dev/null 2>&1 || \
    gh repo create rbaindourov/ella-world --public \
      --description "Standalone Rust 3D multiplayer hangout, Dress to Impress wardrobe & spatial voice server over UDP" \
      --source=. --remote=origin
fi

echo "==> Pushing main branch to $REMOTE_SSH..."
git push -u origin main
echo "✅ Published ELLA World to $REMOTE_SSH"
