#!/usr/bin/env bash
# Roll out the controlled-position experiments built by scripts/experiments.py.
#   Environment: NET (default nets/td-best.net), D (input/output folder, default ../analysis/experiments),
#   IN (folder holding the .in files, default $D), BIN.
cd "$(dirname "$0")"
B=${BIN:-./target/release/mahbusa-rs}
D=${D:-../analysis/experiments}
IN=${IN:-$D}
NET=${NET:-nets/td-best.net}
mkdir -p "$D"
$B evalset --net $NET --games 8000 < $IN/bearoff_stack.in > $D/bearoff_stack.tsv
$B evalset --net $NET --games 3000 < $IN/mother.in > $D/mother.tsv
$B evalset --net $NET --games 2000 < $IN/release_race.in > $D/release_race.tsv
