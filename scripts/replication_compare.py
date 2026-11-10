"""Roll out the guide's example positions with a given net (experiment E4: replication).

    python scripts/replication_compare.py NET OUTFILE [GAMES]

Runs `mahbusa-rs compare` on the example positions of guide §11 and §7: Mistakes 1-4 are rows of
analysis/net1m/quiz_candidates.tsv, Mistakes 5-6 are rows of analysis/net2m/quiz_candidates.tsv
(the same row numbers as in the `quiz_verified.txt` and `examples2_verified.txt` files), plus the two trade-up examples.
"""
import csv
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "engine-rs/target/release/mahbusa-rs"
net, out = sys.argv[1], sys.argv[2]
games = sys.argv[3] if len(sys.argv) > 3 else "1500"

def load(name):
    return list(csv.DictReader(open(ROOT / name), delimiter="\t"))


rows1, rows2 = load("analysis/net1m/quiz_candidates.tsv"), load("analysis/net2m/quiz_candidates.tsv")
cases = []
# (guide label, rows, row number)
for label, rows, i in [("Mistake 1a (6-4, pin on 6)", rows1, 230), ("Mistake 1b (6-4, pin on 3)", rows1, 148),
                       ("Mistake 2 (3-4, keep the pin)", rows1, 69), ("Mistake 3 (6-4, clear the start)", rows1, 151),
                       ("Mistake 4 (4-3, build)", rows1, 165), ("Mistake 4 alt (3-4, build)", rows1, 121),
                       ("Mistake 5 (3-3, counter-pin)", rows2, 3522), ("Mistake 6 (3-1, back checkers)", rows2, 1782)]:
    r = rows[i]
    cases.append((label, f"{r['me']};{r['op']};{r['mpin']};{r['opin']}", r["d1"] + r["d2"]))
for line in open(ROOT / "analysis/pins/tradeup_verified.txt"):
    if line.startswith("== "):
        pos, _, dice = line[3:].strip().rpartition(" dice ")
        cases.append((f"Trade-up example (dice {dice})", pos, dice))

with open(out, "w") as f:
    for label, pos, dice in cases:
        res = subprocess.run([str(BIN), "compare", "--net", net, "--pos", pos, "--dice", dice, "--games", games, "--top", "4"],
                             capture_output=True, text=True, check=True).stdout
        f.write(f"== {label}\n{res}\n")
        f.flush()
