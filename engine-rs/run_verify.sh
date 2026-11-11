#!/usr/bin/env bash
# E2: check one-move judgements against playouts. Needs analysis/experiments/verify/*.in
# (built by scripts/sample_for_verify.py). Writes verify_lift.tsv and verify_quiz.tsv next to them.
cd "$(dirname "$0")"
BIN=${BIN:-./target/release/mahbusa-rs}
NET=${NET:-nets/td-best.net}
D=${D:-../analysis/experiments/verify}
GAMES=${GAMES:-1000}
$BIN verify --net $NET --games $GAMES < $D/verify_lift.in > $D/verify_lift.tsv.part && mv $D/verify_lift.tsv.part $D/verify_lift.tsv
$BIN verify --net $NET --games $GAMES < $D/verify_quiz.in > $D/verify_quiz.tsv.part && mv $D/verify_quiz.tsv.part $D/verify_quiz.tsv
