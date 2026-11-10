#!/usr/bin/env bash
# Train a second, independent network (different seed, separate files) so findings can be
# checked on a net that shares no training history with nets/td-best.net.
# Resumable like run_training.sh: after a container restart just run it again.
#   0-800k games: alpha 0.1   800k-TARGET: alpha 0.03
# Output: nets/alt-latest.net, nets/alt-<N>k.net, nets/alt.log; the finished net is copied to nets/td-alt.net.
set -euo pipefail
cd "$(dirname "$0")"
BIN=${BIN:-./target/release/mahbusa-rs}
TARGET=${TARGET:-1000000}
CHUNK=100000
games_in() { od -An -t u8 -j 20 -N 8 "$1" | tr -d ' '; }
while true; do
  if [ -f nets/alt-latest.net ]; then G=$(games_in nets/alt-latest.net); RESUME="--resume nets/alt-latest.net"; else G=0; RESUME=""; fi
  if [ "$G" -ge "$TARGET" ]; then echo "TARGET REACHED ($G games)"; break; fi
  if [ "$G" -lt 800000 ]; then ALPHA=0.1; else ALPHA=0.03; fi
  $BIN train --out nets/alt $RESUME --games $CHUNK --alpha $ALPHA --threads 4 --checkpoint 100000 --bench-games 500 --seed $((G / 1000 + 5001))
done
cp nets/alt-latest.net nets/td-alt.net
echo "ALL DONE"
