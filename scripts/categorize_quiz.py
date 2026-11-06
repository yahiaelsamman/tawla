"""Sort quiz positions (mahbusa-rs quiz output) into teaching themes.

    python scripts/categorize_quiz.py analysis/net2m/quiz_candidates.tsv [per_category]

For each position it replays the bot's move and the heuristic's move with the Python
engine, measures what changed (pins made or lifted, mother exposure, worst blot,
checkers borne off), and assigns a theme. It prints the biggest examples per theme.
"""
import csv
import sys
from collections import defaultdict

from mahbusa.features import pin_shots
from mahbusa.rules import EGYPTIAN
from mahbusa.state import START, State, _step


def apply(state, notation, dice):
    me, op, mpin, opin = list(state.me), state.op, state.mpin, state.opin
    left = [dice[0]] * 4 if dice[0] == dice[1] else list(dice)
    for st in notation.split():
        a, b = st.split("/")
        a = int(a)
        cands = [a - int(b)] if b != "off" else sorted(d for d in left if d >= a)
        for d in cands:
            if d in left:
                r = _step(me, op, mpin, opin, a, d, EGYPTIAN)
                if r and (b != "off" or r[4] == 0):
                    me, op, mpin, opin = r[0], r[1], r[2], r[3]
                    left.remove(d)
                    break
        else:
            raise ValueError(f"cannot apply {notation} with {dice}")
    return State(me, op, mpin, opin, 0)


def features(before, after):
    f = after.flip()
    blots = [p for p in range(1, 25) if after.me[p] == 1 and not after.i_pin_at(p) and not after.my_pinned_at(p)]
    shots = {p: pin_shots(f, 25 - p) for p in blots}
    new_pins = [p for p in range(1, 25) if after.i_pin_at(p) and not before.i_pin_at(p)]
    lost_pins = [p for p in range(1, 25) if before.i_pin_at(p) and not after.i_pin_at(p)]
    mother = after.me[START] == 1 and not after.my_pinned_at(START)
    return {
        "max_shots": max(shots.values(), default=0),
        "new_pins": new_pins,
        "lost_pins": lost_pins,
        "mother_risk": pin_shots(f, 1) if mother else 0,
        "off": after.me[0] - before.me[0],
    }


def theme(before, fb, fh):
    outside = sum(before.me[7:25])
    pinned = before.opin != 0
    holds_home_pin = any(before.i_pin_at(p) for p in range(1, 7))
    if fh["mother_risk"] > 0 and fb["mother_risk"] == 0:
        return "mother safety"
    if outside == 0 and holds_home_pin:
        return "bear-off with a pin"
    if pinned and fb["new_pins"] and not fh["new_pins"]:
        return "counter-pin when pinned"
    if fb["new_pins"] and not fh["new_pins"]:
        return "take the pin"
    if fh["lost_pins"] and not fb["lost_pins"]:
        return "keep the pin"
    if fh["max_shots"] - fb["max_shots"] >= 5:
        return "blot safety"
    if outside == 0 or fb["off"] or fh["off"]:
        return "bear-off / race"
    return "other"


def main():
    path = sys.argv[1]
    per = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    rows = list(csv.DictReader(open(path), delimiter="\t"))
    groups = defaultdict(list)
    for i, r in enumerate(rows):
        s = State(tuple(map(int, r["me"].split(","))), tuple(map(int, r["op"].split(","))), int(r["mpin"]), int(r["opin"]), 0)
        dice = (int(r["d1"]), int(r["d2"]))
        try:
            fb = features(s, apply(s, r["best"], dice))
            fh = features(s, apply(s, r["heur"], dice))
        except ValueError:
            continue
        groups[theme(s, fb, fh)].append((float(r["loss"]), i, r, fb, fh))
    for name, items in sorted(groups.items(), key=lambda kv: -len(kv[1])):
        items.sort(key=lambda x: -x[0])
        print(f"\n=== {name}: {len(items)} positions")
        for loss, i, r, fb, fh in items[:per]:
            print(f"  row {i} turn {r['turn']} dice {r['d1']}{r['d2']} loss {loss:.2f} | best {r['best']} ({r['best_eq']}) | heur {r['heur']} ({r['heur_eq']})")
            print(f"      best: {fb}\n      heur: {fh}")


if __name__ == "__main__":
    main()
