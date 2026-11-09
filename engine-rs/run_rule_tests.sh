#!/usr/bin/env bash
# Each rule of thumb, played by the net under that constraint, against the plain net.
# Usage: ./run_rule_tests.sh [games]   -> ../analysis/experiments/rules.txt
cd "$(dirname "$0")"
N=${1:-10000}
NET=nets/td-best.net
OUT=../analysis/experiments/rules.txt
[ -n "$APPEND" ] || : > $OUT
for r in ${RULES:-nolift nolift_tradeup alwayspin homepin safe pairs nobury nocrash blocks}; do
  ./target/release/mahbusa-rs bench --a rule:$r:$NET --b $NET --games $N --seed 41 2> /tmp/rule_$r.err >> $OUT
  awk -v r=$r '/RULESTATS/{d+=$4; c+=$6} END{printf "  %s: the rule changed the bot'"'"'s move in %d of %d decisions (%.1f%%)\n", r, c, d, 100*c/d}' /tmp/rule_$r.err >> $OUT
done
echo DONE >> $OUT
