#!/usr/bin/env bash
# Executed on the remote build host (`water` / `solidstate`) by `scripts/remote_build_android.sh`.
set -euo pipefail

export JAVA_HOME="$HOME/android-build/jdk-17"
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$HOME/Android/Sdk"
export ANDROID_NDK_HOME="$HOME/Android/Sdk/ndk/26.3.11579264"
export PATH="$HOME/.cargo/bin:$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/build-tools/34.0.0:$HOME/android-build/bin:$PATH"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"

cd "$HOME/builds/ella-world"

echo "  -> Compiling release ella-world-server..."
cargo build --release --bin ella-world-server

echo "  -> Compiling EllaNativeNet GDExtension (Linux x86_64 + Android arm64-v8a & x86_64)..."
cd "$HOME/builds/ella-world/godot_client/rust_opus"
cargo build --release
cargo ndk -t arm64-v8a -t x86_64 build --release

mkdir -p "$HOME/builds/ella-world/godot_client/bin" "$HOME/builds/ella-world/godot_client/build"
cp -f target/release/libella_native_net.so "$HOME/builds/ella-world/godot_client/bin/libella_native_net.linux.x86_64.so"
cp -f target/aarch64-linux-android/release/libella_native_net.so "$HOME/builds/ella-world/godot_client/bin/libella_native_net.android.arm64.so"
cp -f target/x86_64-linux-android/release/libella_native_net.so "$HOME/builds/ella-world/godot_client/bin/libella_native_net.android.x86_64.so"

cd "$HOME/builds/ella-world/godot_client"

# Ensure clean .gdignore-protected Gradle build template directory for Play Store AAB export
TEMPLATE_DIR="$HOME/.local/share/godot/export_templates/4.3.stable"
mkdir -p android_gradle build rust_opus .godot
: > android_gradle/.gdignore
: > build/.gdignore
: > rust_opus/.gdignore
if [[ -d android ]]; then
  : > android/.gdignore
fi
if [[ ! -f android_gradle/.build_version ]]; then
  echo "  -> Unpacking Godot 4.3 Android Gradle source template for AAB export..."
  mkdir -p android_gradle/build
  unzip -oq "${TEMPLATE_DIR}/android_source.zip" -d android_gradle/build
  printf "4.3.stable" > android_gradle/.build_version
  printf "4.3.stable" > android_gradle/build/.build_version
fi
printf "res://ella_native_net.gdextension\n" > .godot/extension_list.cfg

# Ensure headless Godot editor settings point to JDK 17, Android SDK, and debug keystore
mkdir -p "$HOME/.config/godot"
cat > "$HOME/.config/godot/editor_settings-4.tres" <<EOF_SETTINGS
[gd_resource type="EditorSettings" format=3]

[resource]
export/android/java_sdk_path = "${JAVA_HOME}"
export/android/android_sdk_path = "${ANDROID_HOME}"
export/android/debug_keystore = "$HOME/.android/debug.keystore"
export/android/debug_keystore_user = "androiddebugkey"
export/android/debug_keystore_pass = "android"
EOF_SETTINGS

echo "  -> Importing Godot 4.3 3D GLB meshes, animations & PBR textures..."
godot4 --headless --path "$HOME/builds/ella-world/godot_client" --import < /dev/null

echo "  -> Launching ephemeral ella-world-server for live Godot 4 EWLD UDP integration test..."
TMP_DATA="$(mktemp -d)"
ELLA_HTTP_ADDR="127.0.0.1:24444" \
ELLA_WEBRTC_UDP_PORT="24446" \
ELLA_NATIVE_UDP_PORT="24445" \
ELLA_DATA_DIR="${TMP_DATA}" \
ELLA_PUBLIC_DIR="$HOME/builds/ella-world/public" \
ELLA_SEED_USERS_FILE="$HOME/builds/ella-world/config/users.example.json" \
ELLA_SUPERADMIN_PASSWORD="ci_super_pw_2026" \
ELLA_HMAC_SECRET="ci_hmac_secret_2026" \
"$HOME/builds/ella-world/target/release/ella-world-server" < /dev/null > "${TMP_DATA}/server.log" 2>&1 &
SRV_PID=$!

cleanup_srv() {
  if kill -0 "${SRV_PID}" 2>/dev/null; then
    kill "${SRV_PID}" 2>/dev/null || true
    wait "${SRV_PID}" 2>/dev/null || true
  fi
}
trap cleanup_srv EXIT

for _ in $(seq 1 30); do
  if curl -sf http://127.0.0.1:24444/api/ella-world/health | grep -q '"ok":true'; then
    break
  fi
  sleep 0.2
done

LOGIN_JSON="$(curl -sf -X POST http://127.0.0.1:24444/api/ella-world/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"ella","password":"changeme-starlight","clientId":"godot_ci"}')"
PLAYER_TOK="$(printf '%s' "${LOGIN_JSON}" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')"

echo "  -> Executing Godot 4 headless test suite (GLB + AvatarRig + MainApp + Live EWLD UDP)..."
ELLA_TEST_HOST="127.0.0.1" \
ELLA_TEST_UDP_PORT="24445" \
ELLA_TEST_TOKEN="${PLAYER_TOK}" \
godot4 --headless --path "$HOME/builds/ella-world/godot_client" --script res://tests/run_godot_tests.gd < /dev/null

cleanup_srv
trap - EXIT

echo "  -> Exporting Signed Family Android APK (build/ella-world.apk)..."
godot4 --headless --path "$HOME/builds/ella-world/godot_client" \
  --export-release "Android APK (Signed Family Release)" "build/ella-world.apk" < /dev/null

echo "  -> Verifying APK v2/v3 cryptographic signature & AndroidManifest badging..."
apksigner verify --verbose "$HOME/builds/ella-world/godot_client/build/ella-world.apk"
aapt2 dump badging "$HOME/builds/ella-world/godot_client/build/ella-world.apk" | grep -E "^(package:|sdkVersion:|targetSdkVersion:|application-label:|uses-permission:|native-code:)"

echo "  -> Exporting Google Play Store Android App Bundle (build/ella-world.aab)..."
godot4 --headless --path "$HOME/builds/ella-world/godot_client" \
  --export-release "Android App Bundle (Play Store AAB)" "build/ella-world.aab" < /dev/null

ls -lh "$HOME/builds/ella-world/godot_client/build/ella-world.apk" \
       "$HOME/builds/ella-world/godot_client/build/ella-world.aab"
