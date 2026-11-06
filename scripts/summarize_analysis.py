"""Summarise an analysis directory produced by engine-rs/run_analysis.sh.

    python3 scripts/summarize_analysis.py analysis/net2m

Writes opening_book.tsv and pin_value_clean.tsv into the directory and prints a summary.
"""
import collections
import csv
import sys
from pathlib import Path

D = Path(sys.argv[1])
BOOK = {"21": "24/22 24/23", "31": "24/21 24/23", "41": "24/20 24/23", "51": "24/19 24/23", "61": "24/18 24/23",
        "32": "24/21 24/22", "42": "24/20 24/22", "52": "24/19 24/22", "62": "24/18 24/22", "43": "24/20 24/21",
        "53": "24/19 24/21", "63": "24/18 24/21", "54": "24/19 24/20", "64": "24/18 24/20", "65": "24/18 24/19",
        "11": "24/23 24/23 24/23 24/23", "22": "24/20 24/22 24/22", "33": "24/18 24/21 24/21", "44": "24/16 24/16",
        "55": "24/14 24/19 24/19", "66": "24/12 24/12"}
PEQ = {"21": .042, "31": .037, "41": .070, "51": .043, "61": .097, "32": .050, "42": .065, "52": .066, "62": .106,
       "43": .056, "53": .039, "63": .096, "54": .073, "64": .121, "65": .117, "11": .129, "22": .137, "33": .187,
       "44": .247, "55": .361, "66": .521}
ORDER = list(BOOK)


def norm(mv):
    pos = []
    for st in mv.split():
        a, b = st.split("/")
        if a in pos:
            pos[pos.index(a)] = b
        else:
            pos.append(b)
    return " ".join(sorted(f"24/{p}" for p in pos))


# openings
rows = list(csv.reader(open(D / "opening_rollouts.tsv"), delimiter="\t"))[1:]
by = collections.defaultdict(list)
for r in rows:
    by[r[0]].append(r)
out = ["roll\tbest_by_rollout\tequity\tse\tsecond_best\teq2\tpalamedes_book\tpalamedes_eq\tagrees"]
agree = 0
total_eq = 0.0
for k in ORDER:
    rs = sorted(by[k], key=lambda r: -float(r[3]))
    b = rs[0]
    s = rs[1] if len(rs) > 1 else None
    ok = norm(b[1]) == " ".join(sorted(BOOK[k].split()))
    agree += ok
    total_eq += float(b[3]) * (1 if k[0] == k[1] else 2)
    out.append("\t".join([k, norm(b[1]), f"{float(b[3]):+.3f}", b[4], norm(s[1]) if s else "-",
                          f"{float(s[3]):+.3f}" if s else "-", BOOK[k], f"{PEQ[k]:+.3f}", "yes" if ok else "NO"]))
(D / "opening_book.tsv").write_text("\n".join(out) + "\n")
print("\n".join(out))
print(f"agreement with Palamedes: {agree}/21; first-player equity {total_eq / 36:+.3f} (Palamedes +0.104)\n")

# pin value (raw output reports the pinned side, because the pinner is on roll)
pv = [l.rstrip("\n").split("\t") for l in open(D / "pin_value.tsv")]
base = float(pv[0][0].split(":")[1].split()[0])
lines = ["pin_point\tpinner_equity\tgain\tpinner_double_win%\tpinner_double_loss%"]
for p, e, se, g, dw, dl in pv[2:]:
    lines.append(f"{p}\t{-float(e):+.2f}\t{-float(g):+.2f}\t{dl}\t{dw}")
(D / "pin_value_clean.tsv").write_text("\n".join(lines) + "\n")
print("\n".join(lines))

# replies
rr = list(csv.reader(open(D / "replies_1ply.tsv"), delimiter="\t"))[1:]
n = ok = 0
for first, fm, roll, rep, *_ in rr:
    if roll[0] == roll[1]:
        continue
    n += 1
    ok += set(rep.split()) == {f"24/{24 - int(roll[0])}", f"24/{24 - int(roll[1])}"}
print(f"\nnon-double replies that split two checkers off the start: {ok}/{n}")
