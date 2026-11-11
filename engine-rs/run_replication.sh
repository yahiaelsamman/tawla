#!/usr/bin/env bash
# E4: repeat the main experiments with the independent network (nets/td-alt.net), writing to
# ../analysis/experiments/alt/. Takes about an hour on 4 cores. Resumable: finished steps are skipped.
# Afterwards: python scripts/summarize_replication.py
cd "$(dirname "$0")"
BIN=${BIN:-./target/release/mahbusa-rs}
ALT=nets/td-alt.net
BEST=nets/td-best.net
A=../analysis/experiments/alt
mkdir -p $A
[ -s $A/bench_alt_vs_best.txt ] || $BIN bench --a $ALT --b $BEST --games 20000 --seed 11 > $A/bench_alt_vs_best.txt 2>/dev/null
[ -s $A/bench_alt_vs_heuristic.txt ] || $BIN bench --a $ALT --b heuristic --games 10000 --seed 12 > $A/bench_alt_vs_heuristic.txt 2>/dev/null
[ -s $A/opening_1ply_alt.tsv ] || $BIN opening --net $ALT --games 0 --top 1 > $A/opening_1ply_alt.tsv
[ -s $A/opening_1ply_best.tsv ] || $BIN opening --net $BEST --games 0 --top 1 > $A/opening_1ply_best.tsv
[ -s $A/selfplay_stats.txt ] || $BIN stats --net $ALT --games 20000 > $A/selfplay_stats.txt
[ -s $A/rules.txt ] || { NET=$ALT OUT=$A/rules.txt RULES="nolift nolift_tradeup alwayspin homepin safe pairs nobury nocrash blocks nobury2 safe12 nocrash2" ./run_rule_tests.sh 10000; }
[ -s $A/mistakes_alt.txt ] || python3 ../scripts/replication_compare.py $ALT $A/mistakes_alt.txt 1500
[ -s $A/mistakes_best.txt ] || python3 ../scripts/replication_compare.py $BEST $A/mistakes_best.txt 1500
[ -s $A/pin_value.tsv ] || $BIN pinvalue --net $ALT --games 4000 > $A/pin_value.tsv
[ -s $A/release_race.tsv ] || NET=$ALT D=$A IN=../analysis/experiments ./run_position_tests.sh
echo DONE > $A/replication.done
