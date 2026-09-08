#!/usr/bin/env bash
# Restores the stock bevy_ui_render. Ticket 22. Idempotent.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
MANIFEST="$ROOT/Cargo.toml"

if ! grep -q 'TICKET 22 EXPERIMENT' "$MANIFEST"; then
    echo "not applied — nothing to do"
    exit 0
fi

python3 - "$MANIFEST" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1])
s = p.read_text()
marker = "\n# --- TICKET 22 EXPERIMENT"
assert s.count(marker) == 1, "expected exactly one experiment block"
p.write_text(s[: s.index(marker)].rstrip() + "\n")
PY

echo "reverted — bevy_ui_render is stock 0.19.1 again"
echo "the vendored copy is left in place; delete the directory to remove it"
