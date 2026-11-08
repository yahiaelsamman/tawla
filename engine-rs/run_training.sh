#!/usr/bin/env bash
# Resumable TD(0) training. Each round trains a chunk and resumes from nets/td-latest.net,
# so if it gets interrupted just run this script again.
#   0-1M games: alpha 0.1   1M-2M: alpha 0.03   2M+: alpha 0.01
set -euo pipefail
cd "$(dirname "$0")"
BIN=./target/release/mahbusa-rs
TARGET=${TARGET:-2000000}
CHUNK=100000
games_in() { od -An -t u8 -j 20 -N 8 "$1" | tr -d ' '; }
while true; do
  if [ -f nets/td-latest.net ]; then G=$(games_in nets/td-latest.net); RESUME="--resume nets/td-latest.net"; else G=0; RESUME=""; fi
  if [ "$G" -ge "$TARGET" ]; then echo "TARGET REACHED ($G games)"; break; fi
  if [ "$G" -lt 1000000 ]; then ALPHA=0.1; elif [ "$G" -lt 2000000 ]; then ALPHA=0.03; else ALPHA=0.01; fi
  [ "$G" -ge 1000000 ] && [ ! -f nets/td-stage1.net ] && cp nets/td-latest.net nets/td-stage1.net
  $BIN train --out nets/td $RESUME --games $CHUNK --alpha $ALPHA --threads 4 --checkpoint 25000 --bench-games 1000 --seed $((G / 1000 + 1))
done
cp nets/td-latest.net nets/td-final.net
echo "ALL DONE"
