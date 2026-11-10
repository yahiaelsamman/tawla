"""Build the input for `mahbusa-rs verify` (experiment E2: one-move judgement vs playouts).

    python scripts/sample_for_verify.py OUTDIR

Writes OUTDIR/verify_lift.in (300 keep-or-lift cases) and OUTDIR/verify_quiz.in (100 quiz cases).
Lines: label<TAB>me;op;mpin;opin<TAB>d1d2<TAB>mode. Fixed random seed.
"""
import csv
import gzip
import random
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from categorize_quiz import apply  # noqa: E402
from mahbusa.state import State  # noqa: E402

rng = random.Random(20261110)
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)


def state_of(me, rest):
    op, mpin, opin = rest.split(";")
    return State(tuple(map(int, me.split(","))), tuple(map(int, op.split(","))), int(mpin), int(opin), 0)


def make_line(label, row, mode):
    pos = f"{row['me']};{row['op_mpin_opin']}"
    return f"{label}\t{pos}\t{row['dice']}\t{mode}\n"


rows = list(csv.DictReader(gzip.open(ROOT / "analysis/pins/lift_decisions.tsv.gz", "rt"), delimiter="\t"))
for r in rows:
    r["gain"] = float(r["lift_eq"]) - float(r["keep_eq"])
rng.shuffle(rows)


def makes_deep_pin(r):
    """Does the lifting move make a new pin on the 2-, 3- or 4-point?"""
    s = state_of(r["me"], r["op_mpin_opin"])
    dice = (int(r["dice"][0]), int(r["dice"][1]))
    try:
        after = apply(s, r["lift_move"], dice)
    except ValueError:
        return False
    return any(after.i_pin_at(p) and not s.i_pin_at(p) for p in (2, 3, 4))


chosen, used = [], set()
big = [r for r in rows if r["gain"] >= 0.05][:100]
used.update(id(r) for r in big)
tradeup = []
for r in rows:
    if len(tradeup) == 100:
        break
    if id(r) in used or int(r["pin_point"]) not in (5, 6):
        continue
    if makes_deep_pin(r):
        tradeup.append(r)
used.update(id(r) for r in tradeup)
rest = [r for r in rows if id(r) not in used and r["gain"] < 0.05][:100]

lines = []
for name, group in (("lift_big", big), ("lift_tradeup", tradeup), ("lift_rest", rest)):
    for i, r in enumerate(group):
        lines.append(make_line(f"{name}_{i}", r, f"lift:{r['pin_point']}"))
(out / "verify_lift.in").write_text("".join(lines))
print(f"lift cases: {len(big)} big, {len(tradeup)} trade-up, {len(rest)} rest")

quiz = list(csv.DictReader(open(ROOT / "analysis/net2m/quiz_candidates.tsv"), delimiter="\t"))
sample = rng.sample(quiz, 100)
lines = []
for i, r in enumerate(sample):
    lines.append(f"quiz_{i}\t{r['me']};{r['op']};{r['mpin']};{r['opin']}\t{r['d1']}{r['d2']}\tquiz\n")
(out / "verify_quiz.in").write_text("".join(lines))
print(f"quiz cases: {len(sample)}")
