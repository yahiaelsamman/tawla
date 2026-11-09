#!/usr/bin/env bash
# Roll out the controlled-position experiments built by scripts/experiments.py.
cd "$(dirname "$0")"
B=./target-analysis/release/mahbusa-rs
D=../analysis/experiments
NET=nets/td-best.net
$B evalset --net $NET --games 8000 < $D/bearoff_stack.in > $D/bearoff_stack.tsv
$B evalset --net $NET --games 3000 < $D/mother.in > $D/mother.tsv
$B evalset --net $NET --games 2000 < $D/release_race.in > $D/release_race.tsv
echo DONE > $D/positions.done
