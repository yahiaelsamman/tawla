#!/usr/bin/env bash
# Long look-ahead runs (several hours on 4 cores). Resumable: finished steps are skipped.
#   1. 3-ply vs 2-ply, 4,000 games
#   2. self-play statistics with the 2-ply bot, 10,000 games
#   3. every rule of thumb with both sides at 2-ply, 10,000 games each
cd "$(dirname "$0")"
BIN=${BIN:-./target/release/mahbusa-rs}
NET=nets/td-best.net
L=../analysis/lookahead
[ -s $L/bench_ply3_vs_ply2.txt ] || $BIN bench --a ply3:$NET --b ply2:$NET --games 4000 --seed 61 > $L/bench_ply3_vs_ply2.txt 2>/dev/null
[ -s $L/selfplay_stats_2ply.txt ] || $BIN stats --net $NET --player ply2:$NET --games 10000 > $L/selfplay_stats_2ply.txt
for r in nolift nolift_tradeup pairs homepin alwayspin safe safe12 nobury2 nocrash2 blocks; do
  grep -q "rule2:$r:" $L/rules_2ply_10k.txt 2>/dev/null || BIN=$BIN OUT=$L/rules_2ply_10k.txt APPEND=1 RULES=$r ./run_rule_tests_2ply.sh 10000
done
echo ALL DONE
