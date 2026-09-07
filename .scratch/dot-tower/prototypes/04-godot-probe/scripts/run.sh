#!/usr/bin/env bash
# Runs the probe harness on this desktop.
#
#   ./scripts/run.sh              interactive — drag the sprite, hold keys 1-4
#   ./scripts/run.sh selftest     probe B's automated verdict, prints and exits
#   ./scripts/run.sh shot [path]  writes a reference PNG and exits
#
# Android is a separate script (scripts/android-export.sh) because it needs
# export templates this machine does not have. See README.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

GODOT="${GODOT:-$(command -v godot || true)}"
[[ -n "$GODOT" && -x "$GODOT" ]] || {
    echo "error: godot not found. Install it with: brew install --cask godot" >&2
    echo "       or set GODOT=/path/to/Godot" >&2
    exit 1
}

# The phone shape, so the desktop window stands in for the device. Ticket 02
# settled that layout branches on aspect ratio and never on the platform, which
# is what makes this substitution legitimate rather than a convenience.
RES="480x960"

case "${1:-run}" in
    selftest)
        exec "$GODOT" --path . --resolution "$RES" --position 40,40 -- --selftest
        ;;
    shot)
        OUT="${2:-$ROOT/build/probe-desktop.png}"
        mkdir -p "$(dirname "$OUT")"
        exec "$GODOT" --path . --resolution "$RES" --position 40,40 -- --shot "$OUT"
        ;;
    run)
        echo "keys 1-4 hold synthetic fingers on the ability buttons; drag to move the sprite"
        exec "$GODOT" --path . --resolution "$RES" --position 40,40
        ;;
    *)
        echo "usage: $0 [run|selftest|shot [path]]" >&2
        exit 2
        ;;
esac
