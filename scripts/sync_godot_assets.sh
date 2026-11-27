#!/usr/bin/env bash
# Syncs shared Blender 5.2 GLB packs and CC0 PBR Obby textures from `public/` into `godot_client/assets/`
# so the Godot 4 Android client and Three.js Web client render 100% identical 3D meshes and materials.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_MODELS="${REPO_ROOT}/public/models"
SRC_TEXTURES="${REPO_ROOT}/public/textures/obby"
DST_MODELS="${REPO_ROOT}/godot_client/assets/models"
DST_TEXTURES="${REPO_ROOT}/godot_client/assets/textures/obby"

mkdir -p "${DST_MODELS}" "${DST_TEXTURES}"

for glb in "${SRC_MODELS}"/*.glb; do
  if [[ -f "${glb}" ]]; then
    cp -f "${glb}" "${DST_MODELS}/$(basename "${glb}")"
  fi
done

for tex in "${SRC_TEXTURES}"/*; do
  if [[ -f "${tex}" ]]; then
    cp -f "${tex}" "${DST_TEXTURES}/$(basename "${tex}")"
  fi
done

echo "[sync_godot_assets] Synced $(ls -1 "${DST_MODELS}"/*.glb 2>/dev/null | wc -l) GLB packs and $(ls -1 "${DST_TEXTURES}"/* 2>/dev/null | wc -l) PBR textures into godot_client/assets/."
