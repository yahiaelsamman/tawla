"""Side-by-side numbers for the independent-network replication (experiment E4).

    python scripts/summarize_replication.py > analysis/experiments/replication_numbers.txt

Reads the 2M-net results in analysis/ and the independent-net results in analysis/experiments/alt/
(produced by engine-rs/run_replication.sh).
"""
import csv
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
E = ROOT / "analysis/experiments"
A = E / "alt"


def tsv(path):
    return list(csv.DictReader(open(path), delimiter="\t"))


def rules(path):
    out = {}
    for line in open(path):
        m = re.search(r"rule:(\w+):.*ppg ([+-][\d.]+)", line)
        if m:
            out[m.group(1)] = float(m.group(2))
    return out


print("== Strength")
for name in ("bench_alt_vs_best.txt", "bench_alt_vs_heuristic.txt"):
    if (A / name).exists():
        print((A / name).read_text().strip())

print("\n== Rules of thumb (cost per game, 10,000 games each, 2M net vs independent net)")
r2, ra = rules(E / "rules.txt"), rules(A / "rules.txt")
for k in ("nolift", "nolift_tradeup", "pairs", "homepin", "alwayspin", "safe", "safe12", "nobury2", "nocrash2", "blocks"):
    print(f"{k:16s} {r2.get(k, float('nan')):+.3f}   {ra.get(k, float('nan')):+.3f}")


def mother(path):
    d = {}
    for r in tsv(path):
        m = re.match(r"mother d=(\d+) k=(\d)", r["label"])
        d[(int(m.group(1)), int(m.group(2)))] = -float(r["equity_on_roll"])  # the mother's owner is NOT on roll
    return d


print("\n== Lone mother: cost of leaving one checker instead of two (points)")
m2, ma = mother(E / "mother.tsv"), mother(A / "mother.tsv")
for d in sorted({k[0] for k in m2}):
    print(f"runner {d:2d} away   {m2[(d, 1)] - m2[(d, 2)]:+.2f}   {ma[(d, 1)] - ma[(d, 2)]:+.2f}")


def race(path):
    d = {}
    for r in tsv(path):
        m = re.match(r"race my_pin=(\d) their_pin=(\d) k_me=(\d) k_op=(\d)", r["label"])
        d[tuple(map(int, m.groups()))] = float(r["equity_on_roll"])
    return d


print("\n== Pin standoff (equity for the player on roll)")
q2, qa = race(E / "release_race.tsv"), race(A / "release_race.tsv")
cases = [("equal depth (4,4), stacks 1 v 1", (4, 4, 1, 1)), ("equal depth, you 7 v 1", (4, 4, 7, 1)),
         ("equal depth, you 1 v 7", (4, 4, 1, 7)), ("you deeper (3 v 4), stacks 3 v 3", (3, 4, 3, 3)),
         ("they are deeper (4 v 3), stacks 3 v 3", (4, 3, 3, 3)), ("they are deeper, you 7 v 1", (4, 3, 7, 1))]
for label, key in cases:
    print(f"{label:40s} {q2[key]:+.2f}   {qa[key]:+.2f}")


def bearoff(path):
    d = {}
    for r in tsv(path):
        m = re.match(r"bearoff pin=(\d) (\w+)", r["label"])
        d[(int(m.group(1)), m.group(2))] = float(r["equity_on_roll"])
    return d


print("\n== Bear-off: gain from stacking on the pin")
b2, ba = bearoff(E / "bearoff_stack.tsv"), bearoff(A / "bearoff_stack.tsv")
for p in (3, 4):
    print(f"pin on {p}   {b2[(p, 'stacked')] - b2[(p, 'spread')]:+.2f}   {ba[(p, 'stacked')] - ba[(p, 'spread')]:+.2f}")


def pinvalue(path):
    rows = [l.rstrip("\n").split("\t") for l in open(path)]
    return {int(r[0]): -float(r[3]) for r in rows[2:]}  # pinner's gain vs baseline


print("\n== Pin value by point (pinner's gain, points)")
p2, pa = pinvalue(ROOT / "analysis/net2m/pin_value.tsv"), pinvalue(A / "pin_value.tsv")
for p in (2, 3, 4, 5, 6, 8, 10, 13, 16, 20, 23):
    print(f"pin on {p:2d}   {p2[p]:+.2f}   {pa[p]:+.2f}")

print("\n== Self-play statistics (first lines)")
for name, path in (("2M net", ROOT / "analysis/net2m/selfplay_stats.txt"), ("independent", A / "selfplay_stats.txt")):
    print(name, open(path).readlines()[0].strip())
    print("   ", open(path).readlines()[1].strip())

print("\n== Opening: best 1-ply first move per roll")
o2 = {r["roll"]: r["move"] for r in tsv(A / "opening_1ply_best.tsv")}
oa = {r["roll"]: r["move"] for r in tsv(A / "opening_1ply_alt.tsv")}


def norm(mv):
    pos = []
    for st in mv.split():
        a, b = st.split("/")
        if a in pos:
            pos[pos.index(a)] = b
        else:
            pos.append(b)
    return sorted(pos)


same = [k for k in o2 if norm(o2[k]) == norm(oa[k])]
print(f"same first move for {len(same)} of {len(o2)} rolls; different: " + ", ".join(f"{k} ({o2[k]} vs {oa[k]})" for k in o2 if k not in same))

print("\n== Example positions: best move by playout (net equity | playout equity)")


def best_by_playout(path):
    out, label = {}, None
    for line in open(path):
        if line.startswith("== "):
            label = line[3:].strip()
            out[label] = []
        elif "\t" in line and label:
            mv, net, roll = line.rstrip("\n").split("\t")
            out[label].append((mv, float(roll.split()[1])))
    return out


b2, ba = best_by_playout(A / "mistakes_best.txt"), best_by_playout(A / "mistakes_alt.txt")
for label in b2:
    t2 = max(b2[label], key=lambda x: x[1])
    ta = max(ba[label], key=lambda x: x[1])
    print(f"{label}: 2M best {t2[0]} ({t2[1]:+.2f}) | independent best {ta[0]} ({ta[1]:+.2f})" + ("" if t2[0] == ta[0] else "   <-- differs"))
