#!/usr/bin/env bash
# Each rule of thumb, played by the net under that constraint, against the plain net.
# Usage: ./run_rule_tests.sh [games]
# Environment: RULES (names), APPEND=1 (don't truncate the output), SEED, NET, OUT, BIN.
cd "$(dirname "$0")"
N=${1:-10000}
NET=${NET:-nets/td-best.net}
OUT=${OUT:-../analysis/experiments/rules.txt}
BIN=${BIN:-./target/release/mahbusa-rs}
SEED=${SEED:-41}
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
[ -n "$APPEND" ] || : > "$OUT"
for r in ${RULES:-nolift nolift_tradeup alwayspin homepin safe pairs nobury nocrash blocks}; do
  $BIN bench --a rule:$r:$NET --b $NET --games $N --seed $SEED 2> "$TMP/$r.err" >> "$OUT"
  awk -v r=$r '/RULESTATS/{d+=$4; c+=$6} END{printf "  %s: the rule changed the bot'"'"'s move in %d of %d decisions (%.1f%%)\n", r, c, d, 100*c/d}' "$TMP/$r.err" >> "$OUT"
done
