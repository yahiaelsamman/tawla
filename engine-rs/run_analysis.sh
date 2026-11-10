#!/usr/bin/env bash
# Analysis batch for guide v2. Usage: ./run_analysis.sh nets/td-best.net
set -uo pipefail
cd "$(dirname "$0")"
NET=${1:-nets/td-best.net}
B=${BIN:-./target/release/mahbusa-rs}
OUT=${OUT:-../analysis/net2m}
[ -s $OUT/selfplay_stats.txt ] || $B stats --net $NET --games 20000 > $OUT/selfplay_stats.txt
[ -s $OUT/pin_value.tsv ] || $B pinvalue --net $NET --games 4000 > $OUT/pin_value.tsv
[ -s $OUT/replies_1ply.tsv ] || $B replies --net $NET --games 0 --top 1 > $OUT/replies_1ply.tsv
[ -s $OUT/opening_rollouts.tsv ] || { $B opening --net $NET --games 4000 --top 3 > $OUT/opening_rollouts.tsv.part && mv $OUT/opening_rollouts.tsv.part $OUT/opening_rollouts.tsv; }
[ -s $OUT/releases.txt ] || $B releases --net $NET --games 6000 > $OUT/releases.txt
