"""Pin-shot odds: in how many of the 36 rolls can a checker land exactly n pips away?
Assumes every intermediate landing point is open (no blocks in between)."""
from itertools import product

def reachable(d1, d2):
    if d1 == d2:
        return {d1 * k for k in range(1, 5)}
    return {d1, d2, d1 + d2}

def shots(n):
    return sum(1 for d1, d2 in product(range(1, 7), repeat=2) if n in reachable(d1, d2))

if __name__ == "__main__":
    print("distance  rolls/36  percent")
    for n in range(1, 25):
        s = shots(n)
        if s:
            print(f"{n:8d}  {s:8d}  {100*s/36:6.1f}%")
    print("\nOpening: your blot on point p, opponent's whole stack on your 1-point (distance p-1):")
    for p in range(13, 24):
        n = p - 1
        rolls = [f"{a}-{a}" for a in range(1, 7) if n in reachable(a, a)] + \
                [f"{a}-{b}" for a in range(1, 7) for b in range(a + 1, 7) if n in reachable(a, b)]
        print(f"  point {p:2d}: distance {n:2d}: {shots(n):2d}/36  {', '.join(rolls) or 'unreachable'}")
