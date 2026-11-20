#!/usr/bin/env bash
# The expensive rules of thumb re-tested with look-ahead: the 2-ply bot following the rule vs the
# plain 2-ply bot. About 1.5 games/s on 4 cores, so only large effects can be checked.
# Usage: ./run_rule_tests_2ply.sh [games]   -> ../analysis/lookahead/rules_2ply.txt
cd "$(dirname "$0")"
N=${1:-1500}
NET=${NET:-nets/td-best.net}
OUT=${OUT:-../analysis/lookahead/rules_2ply.txt}
BIN=${BIN:-./target/release/mahbusa-rs}
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
[ -n "$APPEND" ] || : > "$OUT"
for r in ${RULES:-safe nocrash2 nobury2}; do
  $BIN bench --a rule2:$r:$NET --b ply2:$NET --games $N --seed 51 2> "$TMP/$r.err" >> "$OUT"
  awk -v r=$r '/RULESTATS/{d+=$4; c+=$6} END{printf "  %s: the rule changed the 2-ply bot'"'"'s move in %d of %d decisions (%.1f%%)\n", r, c, d, 100*c/d}' "$TMP/$r.err" >> "$OUT"
done
