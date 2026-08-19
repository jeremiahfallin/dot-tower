#!/usr/bin/env bash
# Builds the Rust cdylib for Android and packages it into an APK.
#
# Run scripts/android-setup.sh first — this script assumes the SDK, NDK and JDK
# are already installed and only does work the machine can do unattended.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# scripts/android-setup.sh records ANDROID_HOME / ANDROID_NDK_HOME / JAVA_HOME
# here. Anything already exported in the shell wins.
if [[ -f .env ]]; then
    set -a
    # shellcheck disable=SC1091
    source .env
    set +a
fi

PROFILE="${PROFILE:-debug}"
ABI="arm64-v8a"
TARGET="aarch64-linux-android"
# GameActivity requires API 31+ per android/app/build.gradle's minSdk.
API_LEVEL="${API_LEVEL:-31}"

# ---------------------------------------------------------------------------
# Preflight. Each of these is a thing the human must have done; failing loudly
# here beats a confusing error 200 lines into a Gradle log.
# ---------------------------------------------------------------------------
fail() { echo "error: $*" >&2; exit 1; }

[[ -n "${ANDROID_HOME:-}" ]] || fail "ANDROID_HOME is not set. Run scripts/android-setup.sh."
[[ -d "${ANDROID_HOME}" ]]   || fail "ANDROID_HOME points at ${ANDROID_HOME}, which does not exist."
[[ -n "${ANDROID_NDK_HOME:-}" ]] || fail "ANDROID_NDK_HOME is not set. Run scripts/android-setup.sh."
[[ -d "${ANDROID_NDK_HOME}" ]]   || fail "ANDROID_NDK_HOME points at ${ANDROID_NDK_HOME}, which does not exist."
command -v cargo-ndk >/dev/null || fail "cargo-ndk not installed. Run: cargo install cargo-ndk"
command -v java >/dev/null      || fail "no java on PATH. Run scripts/android-setup.sh."

JNI_DIR="android/app/src/main/jniLibs/${ABI}"
mkdir -p "$JNI_DIR"

# ---------------------------------------------------------------------------
# 1. Rust -> libdot_tower.so
# ---------------------------------------------------------------------------
echo "==> building Rust cdylib for ${TARGET} (${PROFILE})"
RELEASE_FLAG=()
[[ "$PROFILE" == "release" ]] && RELEASE_FLAG=(--release)

cargo ndk \
    --target "$ABI" \
    --platform "$API_LEVEL" \
    --output-dir android/app/src/main/jniLibs \
    build "${RELEASE_FLAG[@]}"

SO="${JNI_DIR}/libdot_tower.so"
[[ -f "$SO" ]] || fail "expected ${SO} but cargo-ndk did not produce it."
echo "    $(du -h "$SO" | cut -f1)  ${SO}"

# ---------------------------------------------------------------------------
# 2. Gradle -> APK
# ---------------------------------------------------------------------------
cd android

# Pass through the NDK version so Gradle's prefab resolution uses the same NDK
# the .so was just built against.
NDK_ARG=()
if [[ -n "${ANDROID_NDK_HOME:-}" ]]; then
    NDK_ARG=(-PndkVersion="$(basename "$ANDROID_NDK_HOME")")
fi

GRADLE_CMD="./gradlew"
[[ -x "$GRADLE_CMD" ]] || GRADLE_CMD="gradle"

TASK="assembleDebug"
[[ "$PROFILE" == "release" ]] && TASK="assembleRelease"

echo "==> ${GRADLE_CMD} ${TASK}"
"$GRADLE_CMD" "$TASK" "${NDK_ARG[@]}"

APK=$(find app/build/outputs/apk -name '*.apk' -type f | head -1)
[[ -n "$APK" ]] || fail "Gradle reported success but produced no APK."
echo "==> APK: android/${APK}"
