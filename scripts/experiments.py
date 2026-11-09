"""Controlled-position experiments for the strategy guide.

    python scripts/experiments.py build DIR      # writes DIR/<exp>.in (inputs for `mahbusa-rs evalset`)

Each experiment changes one thing at a time in an otherwise fixed position.
Positions are written from the point of view of the player ON ROLL ("me").

release_race   two players pinning each other in their home boards; vary how many checkers
               each side has piled on its pin and how deep the pins are.
mother         a lone checker left on the start vs. two or none, with an opponent runner at
               different distances (opponent on roll, so the mother can be hit).
bearoff_stack  bearing off with a pin on your 3-point: checkers stacked on the pin vs spread
               (same pip count).
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))
from mahbusa.state import State  # noqa: E402


def state(me: dict, op: dict, mpin=(), opin=(), me_off=0, op_off=0) -> State:
    m, o = [0] * 25, [0] * 25
    for p, c in me.items():
        m[p] += c
    for p, c in op.items():
        o[p] += c
    m[0] += me_off
    o[0] += op_off
    s = State(m, o, sum(1 << p for p in mpin), sum(1 << q for q in opin), 0)
    s.validate()
    return s


def line(label: str, s: State) -> str:
    return f"{label}\t{','.join(map(str, s.me))};{','.join(map(str, s.op))};{s.mpin};{s.opin}\n"


def spread(n: int, points, base=None) -> dict:
    """Distribute n checkers round-robin over points (first point gets the extras)."""
    d = dict(base or {})
    for i in range(n):
        p = points[i % len(points)]
        d[p] = d.get(p, 0) + 1
    return d


def release_race():
    """me pins them on my point a (k_me checkers on it); they pin me on THEIR point b
    (my point 25-b) with k_op checkers. Everything else is home. Me on roll."""
    out = []
    for a, b in [(4, 4), (4, 3), (3, 4)]:
        for k_me in (1, 2, 3, 5, 7):
            for k_op in (1, 2, 3, 5, 7):
                my_home = [p for p in (6, 5, 4, 3, 2, 1) if p != a]
                their_home = [p for p in (6, 5, 4, 3, 2, 1) if p != b]
                me = spread(15 - 1 - k_me, my_home, {a: k_me, 25 - b: 1})
                op = spread(15 - 1 - k_op, their_home, {b: k_op, 25 - a: 1})
                s = state(me, op, mpin=[a], opin=[b])
                out.append(line(f"race my_pin={a} their_pin={b} k_me={k_me} k_op={k_op}", s))
    return out


def mother():
    """Seen from me (X), opponent on roll. I have k checkers left on my start (k = 0, 1, 2);
    the rest form fixed blocks. The opponent has one runner d pips below my 24-point."""
    out = []
    for d in (2, 6, 8, 9, 12, 14, 17, 18):
        runner = 24 - d  # my numbering
        for k in (0, 1, 2):
            me = {21: 2, 20: 3 + (2 - k), 19: 2, 17: 2, 13: 2, 8: 2}
            if k:
                me[24] = k
            op_mine = {1: 5, 2: 2, 3: 3, 5: 2, 9: 2, runner: 1}  # opponent checkers in MY numbering
            if any(me.get(p) for p in op_mine):
                raise ValueError(f"overlap at d={d}")
            op = {25 - p: c for p, c in op_mine.items()}
            s = state(me, op).flip()  # opponent on roll
            out.append(line(f"mother d={d} k={k}", s))
    return out


def bearoff_stack():
    """Me on roll, all my checkers home, pinning their checker on my point a.
    stacked: 5 on the pin; spread: 1 on the pin with the other 4 moved to a+1 and a-1
    (same pips). They have 14 checkers home waiting."""
    out = []
    for a in (3, 4):
        rest = 15 - 5
        # the other 10 checkers, identical in both variants (so the pip counts match)
        others = {6: 3, 5: 3, 2: 2, 1: 2}
        if a == 4:
            others = {6: 3, 5: 2, 3: 3, 2: 2}
        assert sum(others.values()) == rest
        st = dict(others)
        st[a] = 5
        sp = dict(others)
        sp[a] = 1
        sp[a + 1] = sp.get(a + 1, 0) + 2
        sp[a - 1] = sp.get(a - 1, 0) + 2
        op = {6: 3, 5: 3, 4: 3, 3: 2, 2: 2, 1: 1, 25 - a: 1}
        for label, me in (("stacked", st), ("spread", sp)):
            s = state(me, op, mpin=[a])
            out.append(line(f"bearoff pin={a} {label}", s))
    return out


EXPERIMENTS = {"release_race": release_race, "mother": mother, "bearoff_stack": bearoff_stack}

if __name__ == "__main__":
    if len(sys.argv) < 3 or sys.argv[1] != "build":
        sys.exit(__doc__)
    outdir = Path(sys.argv[2])
    outdir.mkdir(parents=True, exist_ok=True)
    for name, fn in EXPERIMENTS.items():
        lines = fn()
        (outdir / f"{name}.in").write_text("".join(lines))
        print(f"{name}: {len(lines)} positions")
