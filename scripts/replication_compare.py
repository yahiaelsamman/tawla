"""Roll out the guide's example positions with a given net (experiment E4: replication).

    python scripts/replication_compare.py NET OUTFILE [GAMES]

Runs `mahbusa-rs compare` on the six Mistake positions of guide §11 (rows of
analysis/net2m/quiz_candidates.tsv), the two follow-up examples, and the two trade-up examples.
"""
import csv
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "engine-rs/target/release/mahbusa-rs"
net, out = sys.argv[1], sys.argv[2]
games = sys.argv[3] if len(sys.argv) > 3 else "1500"

rows = list(csv.DictReader(open(ROOT / "analysis/net2m/quiz_candidates.tsv"), delimiter="\t"))
cases = []
# (guide label, row of quiz_candidates.tsv)
for label, i in [("Mistake 1a (6-4, pin on 6)", 230), ("Mistake 1b (6-4, pin on 3)", 148), ("Mistake 2 (3-4, keep the pin)", 69),
                 ("Mistake 3 (6-4, clear the start)", 151), ("Mistake 4 (4-3, build)", 165), ("Mistake 4 alt (3-4, build)", 121),
                 ("Mistake 5 (3-3, counter-pin)", 3522), ("Mistake 6 (3-1, back checkers)", 1782)]:
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
