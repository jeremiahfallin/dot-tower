#!/usr/bin/env bash
# Exports and installs the probe harness on a connected Android device.
#
# NOT YET RUN. This is written against the preset in export_presets.cfg and is
# blocked on one thing only: Godot's export templates, a ~1GB download this
# machine does not have. Everything else it needs is already present, having
# been installed for the Bevy bringup — SDK build-tools 36, platform android-36,
# and ~/.android/debug.keystore.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Reuse the toolchain paths the Bevy bringup recorded, so there is one source of
# truth for where the SDK lives.
REPO_ROOT="$(cd "$ROOT/../../../.." && pwd)"
if [[ -f "$REPO_ROOT/.env" ]]; then
    set -a
    # shellcheck disable=SC1091
    source "$REPO_ROOT/.env"
    set +a
fi

GODOT="${GODOT:-$(command -v godot || true)}"
[[ -n "$GODOT" && -x "$GODOT" ]] || { echo "error: godot not found." >&2; exit 1; }

VERSION="$("$GODOT" --version | head -1 | cut -d. -f1-3)"
TEMPLATES="$HOME/Library/Application Support/Godot/export_templates/${VERSION}.stable"
if [[ ! -d "$TEMPLATES" ]]; then
    cat >&2 <<MSG
error: no export templates for ${VERSION}.

  Godot cannot produce an APK without them. Either:
    - open Godot and use Editor > Manage Export Templates > Download, or
    - curl -fL -o /tmp/t.tpz \\
        https://github.com/godotengine/godot/releases/download/${VERSION}-stable/Godot_v${VERSION}-stable_export_templates.tpz
      then unzip it so that its 'templates' directory becomes:
        $TEMPLATES

  It is roughly 1GB.
MSG
    exit 1
fi

# Godot reads these from editor settings, not the environment, so a CLI export
# on a machine whose editor has never been configured fails with an unhelpful
# message. Write them once, idempotently.
SETTINGS="$HOME/Library/Application Support/Godot/editor_settings-$(echo "$VERSION" | cut -d. -f1-2).tres"
if [[ -f "$SETTINGS" && -n "${ANDROID_HOME:-}" ]]; then
    python3 - "$SETTINGS" "$ANDROID_HOME" "${JAVA_HOME:-}" <<'PY'
import sys, pathlib
path, sdk, jdk = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
want = {
    "export/android/android_sdk_path": sdk,
    "export/android/debug_keystore": str(pathlib.Path.home() / ".android/debug.keystore"),
    "export/android/debug_keystore_user": "androiddebugkey",
    "export/android/debug_keystore_pass": "android",
}
if jdk:
    want["export/android/java_sdk_path"] = jdk
lines = path.read_text().splitlines()
out, seen = [], set()
for line in lines:
    key = line.split(" = ")[0].strip()
    if key in want:
        out.append(f'{key} = "{want[key]}"')
        seen.add(key)
    else:
        out.append(line)
for key, val in want.items():
    if key not in seen:
        out.append(f'{key} = "{val}"')
path.write_text("\n".join(out) + "\n")
print(f"configured {len(want)} android export settings")
PY
fi

ADB="${ANDROID_HOME:-}/platform-tools/adb"
[[ -x "$ADB" ]] || ADB="$(command -v adb || true)"

mkdir -p build
APK="$ROOT/build/dot-tower-probe.apk"
echo "==> exporting $APK"
"$GODOT" --headless --path . --export-debug "Android" "$APK"

if [[ -n "$ADB" && -x "$ADB" ]] && [[ -n "$("$ADB" devices | awk 'NR>1 && $2=="device"')" ]]; then
    echo "==> installing"
    "$ADB" install -r "$APK"
    "$ADB" logcat -c || true
    "$ADB" shell monkey -p dev.dottower.godotprobe -c android.intent.category.LAUNCHER 1 >/dev/null
    echo
    echo "=== probe log (ctrl-c to stop) ==="
    "$ADB" logcat -s godot GodotEngine
else
    echo "no authorised device; APK is at $APK"
fi
