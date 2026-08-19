#!/usr/bin/env bash
# Builds, installs and launches on a connected device, then tails the probe log.
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

APP_ID="dev.dottower"
ADB="${ANDROID_HOME:-}/platform-tools/adb"
[[ -x "$ADB" ]] || ADB="$(command -v adb || true)"
[[ -n "$ADB" && -x "$ADB" ]] || { echo "error: adb not found. Run scripts/android-setup.sh." >&2; exit 1; }

DEVICES=$("$ADB" devices | awk 'NR>1 && $2=="device" {print $1}')
if [[ -z "$DEVICES" ]]; then
    echo "error: no authorised device." >&2
    echo "  - plug the phone in over USB" >&2
    echo "  - accept the 'Allow USB debugging?' prompt on the phone" >&2
    echo "  - re-check with: $ADB devices" >&2
    exit 1
fi
echo "==> device: $(echo "$DEVICES" | head -1)"

./scripts/android-build.sh

APK=$(find android/app/build/outputs/apk -name '*.apk' -type f | head -1)
echo "==> installing ${APK}"
"$ADB" install -r "$APK"

# Clear first so the probe output isn't buried in the boot spam.
"$ADB" logcat -c || true
echo "==> launching ${APP_ID}"
"$ADB" shell monkey -p "$APP_ID" -c android.intent.category.LAUNCHER 1 >/dev/null

echo
echo "=== probe log (ctrl-c to stop) ======================================="
echo "Probes A-F are described at the top of src/lib.rs."
echo "To test probe E, background the app with the home button and watch for"
echo "'probe E: saved in Nms'. If nothing appears, the suspend save is not"
echo "landing and ticket 12's design needs revisiting."
echo "======================================================================"
"$ADB" logcat -v time dot-tower:V RustStdoutStderr:V android-activity:V "*:S"
