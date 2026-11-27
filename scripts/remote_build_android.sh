#!/usr/bin/env bash
# Offloads the complete Ella's World Rust + GDExtension (Linux x86_64 & Android arm64/x86_64) +
# Headless Godot 4.3 Asset Import, Live Ephemeral UDP Integration Test, Signed APK Export, and
# Google Play Store AAB Export onto `water` (`robert@192.168.11.7`, fallback `solidstate`).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

select_remote_host() {
  if [[ -n "${ELLA_BUILD_HOST:-}" ]]; then
    echo "${ELLA_BUILD_HOST}"
    return
  fi
  if ssh -o BatchMode=yes -o ConnectTimeout=3 robert@192.168.11.7 "true" 2>/dev/null; then
    echo "robert@192.168.11.7"
    return
  fi
  echo "robert@192.168.11.25"
}

HOST="$(select_remote_host)"
REMOTE_DIR="~/builds/ella-world"

echo "🔄 [1/5] Syncing shared Blender 5.2 GLB & PBR assets into godot_client/assets/..."
./scripts/sync_godot_assets.sh

echo "🚀 [2/5] Syncing workspace to remote build node (${HOST}:${REMOTE_DIR})..."
ssh -o BatchMode=yes "${HOST}" "mkdir -p ${REMOTE_DIR}"
rsync -az \
  --exclude '.git/' \
  --exclude '/target/' \
  --exclude '/data/' \
  --exclude '/scratch/' \
  --exclude '/godot_client/.godot/' \
  --exclude '/godot_client/android/' \
  --exclude '/godot_client/build/' \
  --exclude '/godot_client/bin/*.so' \
  --exclude '/godot_client/rust_opus/target/' \
  --exclude '/public/download/*.apk' \
  --exclude '/public/download/*.aab' \
  "${REPO_ROOT}/" "${HOST}:${REMOTE_DIR}/"

echo "🛠️  [3/5] Building ella-world-server, EllaNativeNet GDExtension, running Godot 4 headless tests & exporting APK/AAB on ${HOST}..."
ssh -o BatchMode=yes "${HOST}" "bash ${REMOTE_DIR}/scripts/remote_android_worker.sh"

echo "📥 [4/5] Pulling signed APK, Play Store AAB & release server binary from ${HOST}..."
mkdir -p "${REPO_ROOT}/public/download" "${REPO_ROOT}/godot_client/build" "${REPO_ROOT}/target/release"
rsync -az "${HOST}:${REMOTE_DIR}/godot_client/build/ella-world.apk" "${REPO_ROOT}/public/download/ella-world.apk"
rsync -az "${HOST}:${REMOTE_DIR}/godot_client/build/ella-world.aab" "${REPO_ROOT}/godot_client/build/ella-world.aab"
rsync -az "${HOST}:${REMOTE_DIR}/target/release/ella-world-server" "${REPO_ROOT}/target/release/ella-world-server"
chmod +x "${REPO_ROOT}/target/release/ella-world-server"

echo "📝 [5/5] Writing public/download/ella-world-release.json metadata..."
APK_BYTES="$(stat -c '%s' "${REPO_ROOT}/public/download/ella-world.apk")"
APK_SHA256="$(sha256sum "${REPO_ROOT}/public/download/ella-world.apk" | awk '{print $1}')"
AAB_BYTES="$(stat -c '%s' "${REPO_ROOT}/godot_client/build/ella-world.aab")"
AAB_SHA256="$(sha256sum "${REPO_ROOT}/godot_client/build/ella-world.aab" | awk '{print $1}')"
BUILT_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

cat > "${REPO_ROOT}/public/download/ella-world-release.json" <<EOF_META
{
  "ok": true,
  "available": true,
  "appName": "Ella's World",
  "applicationId": "com.robertscreations.ellaworld",
  "packageName": "com.robertscreations.ellaworld",
  "versionName": "1.0.0",
  "versionCode": 1,
  "engine": "Godot 4.3 Mobile (Vulkan + GL Compatibility) + EllaNativeNet (Rust GDExtension EWLD UDP + RFC 6716 Opus)",
  "minSdk": 21,
  "targetSdk": 34,
  "architectures": ["arm64-v8a", "x86_64"],
  "downloadUrl": "/download/ella-world.apk",
  "apkBytes": ${APK_BYTES},
  "sizeBytes": ${APK_BYTES},
  "sha256": "${APK_SHA256}",
  "aabConfigured": true,
  "playStoreBundle": {
    "format": "aab",
    "minSdk": 26,
    "targetSdk": 34,
    "sizeBytes": ${AAB_BYTES},
    "sha256": "${AAB_SHA256}"
  },
  "builtAt": "${BUILT_AT}"
}
EOF_META

echo "✅ Android APK (${APK_BYTES} bytes, sha256=${APK_SHA256}) & Play Store AAB (${AAB_BYTES} bytes) ready!"
