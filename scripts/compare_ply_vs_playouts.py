"""Compare 1-ply and 2-ply judgements with the 1,000-game playouts in analysis/experiments/verify/.

    python scripts/compare_ply_vs_playouts.py analysis/lookahead > analysis/lookahead/verify_vs_2ply.txt

Reads DIR/verify_lift_2ply.tsv and DIR/verify_quiz_2ply.tsv, which are the output of
`mahbusa-rs verify --games 0 --ply2 1` on the same inputs as the playout check.
"""
import csv, math, sys
V = "analysis/experiments/verify"
S = sys.argv[1]
res = {}
for name in ("lift", "quiz"):
    roll = {r["label"]: r for r in csv.DictReader(open(f"{V}/verify_{name}.tsv"), delimiter="\t")}
    for r in csv.DictReader(open(f"{S}/verify_{name}_2ply.tsv"), delimiter="\t"):
        o = roll[r["label"]]
        assert o["moveA"] == r["moveA"] and o["moveB"] == r["moveB"], r["label"]
        res[r["label"]] = (o, r)
def grp(l): return l.rsplit("_", 1)[0]
groups = {}
for l, v in res.items(): groups.setdefault(grp(l), []).append(v)
groups["ALL"] = list(res.values())
print(f"{'group':14s} {'n':>4s} {'decided':>7s} | {'1-ply right':>11s} {'2-ply right':>11s} | {'|err| of A-B, 1-ply':>20s} {'2-ply':>7s} | {'|err| per move, 1-ply':>21s} {'2-ply':>7s}")
for g, vs in groups.items():
    dec = r1 = r2 = 0; e1 = e2 = m1 = m2 = 0.0
    for o, p in vs:
        D = float(o["rollA"]) - float(o["rollB"]); se = math.hypot(float(o["seA"]), float(o["seB"]))
        d1 = float(o["netA"]) - float(o["netB"]); d2 = float(p["ply2A"]) - float(p["ply2B"])
        e1 += abs(d1 - D); e2 += abs(d2 - D)
        m1 += (abs(float(o["netA"]) - float(o["rollA"])) + abs(float(o["netB"]) - float(o["rollB"]))) / 2
        m2 += (abs(float(p["ply2A"]) - float(o["rollA"])) + abs(float(p["ply2B"]) - float(o["rollB"]))) / 2
        if abs(D) >= 2 * se:
            dec += 1; r1 += (d1 > 0) == (D > 0); r2 += (d2 > 0) == (D > 0)
    n = len(vs)
    print(f"{g:14s} {n:4d} {dec:7d} | {r1:5d} ({100*r1/dec:3.0f}%) {r2:5d} ({100*r2/dec:3.0f}%) | {e1/n:20.3f} {e2/n:7.3f} | {m1/n:21.3f} {m2/n:7.3f}")
