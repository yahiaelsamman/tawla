"""Summarise `mahbusa-rs verify` output (experiment E2).

    python scripts/summarize_verify.py analysis/experiments/verify > analysis/experiments/verify_summary.txt

Each row compares move A (keep / the net's choice) with move B (lift / the heuristic's choice)
by the net's one-move value and by playouts. A case counts as "agree" if the playout prefers the
same move as the one-move value, "disagree" if it prefers the other, and "too close" if the
playout difference is under 2 combined standard errors.
"""
import csv
import math
import sys
from collections import defaultdict
from pathlib import Path

D = Path(sys.argv[1])
rows = []
for name in ("verify_lift.tsv", "verify_quiz.tsv"):
    p = D / name
    if p.exists():
        rows += list(csv.DictReader(open(p), delimiter="\t"))


def group(label):
    return label.rsplit("_", 1)[0]


def bucket(margin):
    return "<0.10" if margin < 0.10 else "0.10-0.25" if margin < 0.25 else ">=0.25"


def classify(r):
    na, nb = float(r["netA"]), float(r["netB"])
    ra, rb = float(r["rollA"]), float(r["rollB"])
    se = math.hypot(float(r["seA"]), float(r["seB"]))
    one_prefers_a = na >= nb
    diff = ra - rb
    if abs(diff) < 2 * se:
        return "close", abs(na - nb)
    return ("agree" if (diff > 0) == one_prefers_a else "disagree"), abs(na - nb)


by_group = defaultdict(list)
for r in rows:
    by_group[group(r["label"])].append(r)

print("ONE-MOVE JUDGEMENT vs PLAYOUTS (1,000 playouts per move; net on both sides)")
print("agree = playout prefers the same move as the one-move value; close = playout difference under 2 standard errors\n")
for g, rs in by_group.items():
    cl = [classify(r) for r in rs]
    n = len(rs)
    cnt = lambda k: sum(1 for c, _ in cl if c == k)
    print(f"{g}: {n} cases   agree {cnt('agree')}   disagree {cnt('disagree')}   close {cnt('close')}")
    decided = cnt("agree") + cnt("disagree")
    if decided:
        print(f"    of the {decided} decided cases, the one-move value was right in {100 * cnt('agree') / decided:.0f}%")
    for b in ("<0.10", "0.10-0.25", ">=0.25"):
        sub = [c for c, m in cl if bucket(m) == b]
        if sub:
            a, d = sub.count("agree"), sub.count("disagree")
            print(f"    one-move margin {b:>9}: {len(sub):3d} cases, agree {a}, disagree {d}, close {sub.count('close')}" +
                  (f"  (right in {100 * a / (a + d):.0f}% of decided)" if a + d else ""))
    mean_one = sum(float(r["netA"]) - float(r["netB"]) for r in rs) / n
    mean_roll = sum(float(r["rollA"]) - float(r["rollB"]) for r in rs) / n
    print(f"    mean (A - B): one-move {mean_one:+.3f}, playout {mean_roll:+.3f}")
    if g.startswith("lift"):
        one_lift = sum(1 for r in rs if float(r["netB"]) > float(r["netA"]))
        roll_lift = sum(1 for r in rs if float(r["rollB"]) > float(r["rollA"]))
        sig_lift = sum(1 for r in rs if float(r["rollB"]) - float(r["rollA"]) > 2 * math.hypot(float(r["seA"]), float(r["seB"])))
        sig_keep = sum(1 for r in rs if float(r["rollA"]) - float(r["rollB"]) > 2 * math.hypot(float(r["seA"]), float(r["seB"])))
        print(f"    lifting is better: by one-move value in {one_lift}/{n}, by playout point estimate in {roll_lift}/{n}; "
              f"clearly (2 SE) lifting in {sig_lift}, clearly keeping in {sig_keep}")
    print()
