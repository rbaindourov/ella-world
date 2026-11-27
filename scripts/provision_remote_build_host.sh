#!/usr/bin/env bash
# Provisions user-space Rust, cargo-ndk, OpenJDK 17, Android SDK/NDK 34, and Godot 4.3 + Export Templates
# on the remote compute node (default: water / robert@192.168.11.7, fallback: solidstate / robert@192.168.11.25).
# Strictly additive & idempotent (zero root/Docker required, zero destructive commands).
set -euo pipefail

select_remote_host() {
  if [ -n "${ELLA_REMOTE_BUILD_HOST:-}" ]; then
    echo "$ELLA_REMOTE_BUILD_HOST"
    return
  fi
  for candidate in "robert@192.168.11.7" "robert@192.168.11.25"; do
    if ssh -o BatchMode=yes -o ConnectTimeout=3 "$candidate" "true" 2>/dev/null; then
      echo "$candidate"
      return
    fi
  done
  echo "ERROR: Neither water (192.168.11.7) nor solidstate (192.168.11.25) is reachable via SSH." >&2
  exit 1
}

REMOTE_HOST="$(select_remote_host)"
echo "==> Provisioning native Rust + Android + Godot 4.3 toolchain on ${REMOTE_HOST}..."

ssh -o BatchMode=yes "$REMOTE_HOST" 'bash -s' <<'EOF'
set -euo pipefail

mkdir -p "$HOME/android-build/bin" "$HOME/android-build/downloads" "$HOME/Android/Sdk"

# 1. Rustup + Cargo + Android NDK targets
if [ ! -x "$HOME/.cargo/bin/cargo" ]; then
  echo "==> [1/5] Installing rustup (stable) in user space..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile default
fi
export PATH="$HOME/.cargo/bin:$HOME/android-build/bin:$PATH"
rustup component add rustfmt clippy
rustup target add aarch64-linux-android x86_64-linux-android

# 2. cargo-ndk
if [ ! -x "$HOME/.cargo/bin/cargo-ndk" ] && [ ! -x "$HOME/android-build/bin/cargo-ndk" ]; then
  echo "==> [2/5] Installing cargo-ndk via cargo..."
  CARGO_BUILD_JOBS=8 cargo install cargo-ndk --locked
fi

# 3. Portable OpenJDK 17 (required by Godot 4.3 Gradle / Android build tools)
if [ ! -x "$HOME/android-build/jdk-17/bin/javac" ]; then
  echo "==> [3/5] Installing portable Eclipse Temurin OpenJDK 17..."
  JDK_TAR="$HOME/android-build/downloads/temurin-jdk17.tar.gz"
  curl -fsSL "https://api.adoptium.net/v3/binary/latest/17/ga/linux/x64/jdk/hotspot/normal/eclipse" -o "$JDK_TAR"
  mkdir -p "$HOME/android-build/jdk-17"
  tar -xzf "$JDK_TAR" --strip-components=1 -C "$HOME/android-build/jdk-17"
fi
export JAVA_HOME="$HOME/android-build/jdk-17"
export PATH="$JAVA_HOME/bin:$PATH"

# 4. Android SDK Command-line Tools + Platform 34 + Build-Tools 34.0.0 + NDK 26.3
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$HOME/Android/Sdk"
if [ ! -x "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" ]; then
  echo "==> [4/5] Installing Android SDK command-line tools..."
  CMD_ZIP="$HOME/android-build/downloads/commandlinetools-linux.zip"
  curl -fsSL "https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip" -o "$CMD_ZIP"
  mkdir -p "$ANDROID_HOME/cmdline-tools"
  unzip -qo "$CMD_ZIP" -d "$ANDROID_HOME/cmdline-tools"
  if [ -d "$ANDROID_HOME/cmdline-tools/cmdline-tools" ] && [ ! -d "$ANDROID_HOME/cmdline-tools/latest" ]; then
    mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
  fi
fi

SDKMANAGER="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
yes | "$SDKMANAGER" --licenses >/dev/null 2>&1 || true
if [ ! -d "$ANDROID_HOME/platforms/android-34" ] || [ ! -d "$ANDROID_HOME/build-tools/34.0.0" ] || [ ! -d "$ANDROID_HOME/ndk/26.3.11579264" ]; then
  echo "==> [4b/5] Installing Android Platform 34, Build-Tools 34.0.0, and NDK 26.3.11579264..."
  "$SDKMANAGER" --install "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;26.3.11579264"
fi

# 5. Godot 4.3-stable Headless Binary + Official Export Templates
GODOT_BIN="$HOME/android-build/bin/godot4"
TEMPLATES_DIR="$HOME/.local/share/godot/export_templates/4.3.stable"
if [ ! -x "$GODOT_BIN" ]; then
  echo "==> [5/5] Installing Godot 4.3-stable Linux x86_64 binary..."
  GODOT_ZIP="$HOME/android-build/downloads/Godot_v4.3-stable_linux.x86_64.zip"
  curl -fsSL "https://github.com/godotengine/godot/releases/download/4.3-stable/Godot_v4.3-stable_linux.x86_64.zip" -o "$GODOT_ZIP"
  unzip -qo "$GODOT_ZIP" -d "$HOME/android-build/bin"
  cp -f "$HOME/android-build/bin/Godot_v4.3-stable_linux.x86_64" "$GODOT_BIN"
  chmod +x "$GODOT_BIN"
fi

if [ ! -f "$TEMPLATES_DIR/android_release.apk" ] || [ ! -f "$TEMPLATES_DIR/android_source.zip" ]; then
  echo "==> [5b/5] Installing Godot 4.3-stable Export Templates..."
  TPZ_FILE="$HOME/android-build/downloads/Godot_v4.3-stable_export_templates.tpz"
  curl -fsSL "https://github.com/godotengine/godot/releases/download/4.3-stable/Godot_v4.3-stable_export_templates.tpz" -o "$TPZ_FILE"
  mkdir -p "$HOME/android-build/downloads/tpz_extract" "$TEMPLATES_DIR"
  unzip -qo "$TPZ_FILE" -d "$HOME/android-build/downloads/tpz_extract"
  cp -rf "$HOME/android-build/downloads/tpz_extract/templates/"* "$TEMPLATES_DIR/"
fi

# Configure Godot 4 editor settings so headless Android APK & AAB exports know SDK & JDK paths
mkdir -p "$HOME/.config/godot"
cat > "$HOME/.config/godot/editor_settings-4.tres" <<GDSET
[gd_resource type="EditorSettings" format=3]

[resource]
export/android/android_sdk_path = "$ANDROID_HOME"
export/android/java_sdk_path = "$JAVA_HOME"
export/android/debug_keystore = "$HOME/.android/debug.keystore"
export/android/debug_keystore_user = "androiddebugkey"
export/android/debug_keystore_pass = "android"
GDSET

if [ ! -f "$HOME/.android/debug.keystore" ]; then
  mkdir -p "$HOME/.android"
  "$JAVA_HOME/bin/keytool" -genkeypair -v \
    -keystore "$HOME/.android/debug.keystore" \
    -storepass android -alias androiddebugkey -keypass android \
    -keyalg RSA -keysize 2048 -validity 10000 \
    -dname "CN=Android Debug,O=Android,C=US" >/dev/null 2>&1
fi

if [ ! -f "$HOME/android-build/ella-world-release.keystore" ]; then
  "$JAVA_HOME/bin/keytool" -genkeypair -v \
    -keystore "$HOME/android-build/ella-world-release.keystore" \
    -storepass ellaworld2026 -alias ellaworld -keypass ellaworld2026 \
    -keyalg RSA -keysize 2048 -validity 10000 \
    -dname "CN=Ella World Family Release,OU=RobertsCreations,O=RobertsCreations,L=LosAngeles,ST=CA,C=US" >/dev/null 2>&1
fi

echo "✅ Remote toolchain ready:"
rustc --version
cargo --version
cargo ndk --version
"$JAVA_HOME/bin/java" -version 2>&1 | head -n 2
"$GODOT_BIN" --version
ls -lh "$TEMPLATES_DIR/android_release.apk" "$TEMPLATES_DIR/android_source.zip"
EOF
