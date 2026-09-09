#!/usr/bin/env bash
# Points the build at the patched bevy_ui_render. Ticket 22.
# Reverse with ./revert.sh. Idempotent.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
MANIFEST="$ROOT/Cargo.toml"

if grep -q '^\[patch.crates-io\]' "$MANIFEST"; then
    echo "already applied — nothing to do"
    exit 0
fi

cat >> "$MANIFEST" <<'TOML'

# --- TICKET 22 EXPERIMENT — remove with prototypes/22-ui-vertex-alignment/revert.sh
# Points bevy_ui_render at a vendored 0.19.1 whose UI vertex layout is 16-byte
# aligned. See .scratch/dot-tower/prototypes/22-ui-vertex-alignment/README.md.
[patch.crates-io]
bevy_ui_render = { path = ".scratch/dot-tower/prototypes/22-ui-vertex-alignment/bevy_ui_render" }
TOML

echo "applied — bevy_ui_render now resolves to the patched copy"
echo "next: ./scripts/android-run.sh"
