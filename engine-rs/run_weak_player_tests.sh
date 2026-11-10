#!/usr/bin/env bash
# Each rule of thumb applied to the simple rule-based (heuristic) bot, against the plain heuristic bot.
# Shows whether a rule helps a weaker player, not only whether it costs the strong one.
# Usage: ./run_weak_player_tests.sh [games]   -> ../analysis/experiments/rules_weak_player.txt
cd "$(dirname "$0")"
N=${1:-20000}
OUT=${OUT:-../analysis/experiments/rules_weak_player.txt}
BIN=${BIN:-./target/release/mahbusa-rs}
SEED=${SEED:-43}
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
: > "$OUT"
for r in ${RULES:-nolift nolift_tradeup pairs homepin alwayspin safe safe12 nobury2 nocrash2}; do
  $BIN bench --a hrule:$r --b heuristic --games $N --seed $SEED 2> "$TMP/$r.err" >> "$OUT"
  awk -v r=$r '/RULESTATS/{d+=$4; c+=$6} END{printf "  %s: the rule changed the heuristic bot'"'"'s move in %d of %d decisions (%.1f%%)\n", r, c, d, 100*c/d}' "$TMP/$r.err" >> "$OUT"
done
