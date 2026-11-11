#!/usr/bin/env bash
# Checks that the browser engine (web/engine.js + web/net.js) agrees with the Rust engine:
# same legal moves, same chosen move and equity, and the same network weights.
# Needs: node, python3 with the package installed (or PYTHONPATH=src), and a built Rust engine.
#   web/tests/run_engine_check.sh [N_GAMES]   (about 100 random games give about 5,000 positions)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
BIN=${BIN:-$ROOT/engine-rs/target/release/mahbusa-rs}
NET=$ROOT/engine-rs/nets/td-best.net
GAMES=${1:-100}
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
PY=${PYTHON:-python3}
[ -x "$ROOT/.venv/bin/python" ] && PY="$ROOT/.venv/bin/python"
"$PY" "$ROOT/web/tests/make_cases.py" "$GAMES" "$TMP/cases.txt"
"$BIN" movegen < "$TMP/cases.txt" > "$TMP/moves.txt"
"$BIN" choose --net "$NET" < "$TMP/cases.txt" > "$TMP/choose.txt"
node "$ROOT/web/tests/check_net_embed.js" "$ROOT/web/net.js" "$NET"
node "$ROOT/web/tests/check_engine.js" "$TMP/cases.txt" "$TMP/moves.txt" "$TMP/choose.txt" "$NET"
