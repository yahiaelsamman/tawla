"""Position features used by heuristic bots and, later, by the neural network."""
from __future__ import annotations

from .state import START, State, _can_land

# Ordered rolls out of 36: doubles weigh 1, non-doubles 2.
ROLLS = [(a, b, 1 if a == b else 2) for a in range(1, 7) for b in range(a, 7)]


def _can_reach(state: State, src: int, target: int, d1: int, d2: int) -> bool:
    """Can the mover bring the checker on src (mover's numbering) to land on target
    using roll (d1, d2)? Intermediate landing points must be open. Forced-move
    subtleties are ignored (this is a shot-counting estimate, as players count)."""
    me, op, opin = state.me, state.op, state.opin
    dist = src - target
    if dist <= 0:
        return False
    if d1 == d2:
        if dist % d1 or dist // d1 > 4:
            return False
        for k in range(1, dist // d1 + 1):
            if not _can_land(me, op, opin, src - k * d1):
                return False
        return True
    if dist == d1 or dist == d2:
        return _can_land(me, op, opin, target)
    if dist == d1 + d2:
        if not _can_land(me, op, opin, target):
            return False
        return _can_land(me, op, opin, src - d1) or _can_land(me, op, opin, src - d2)
    return False


def pin_shots(state: State, target: int) -> int:
    """Rolls out of 36 with which the player to move can land on their point `target`
    (1..24) with some movable checker. Use it on a lone enemy checker to count how
    often it can be pinned."""
    me = state.me
    sources = [p for p in range(target + 1, 25) if me[p] and not state.my_pinned_at(p)]
    if not sources:
        return 0
    total = 0
    for d1, d2, w in ROLLS:
        if any(_can_reach(state, s, target, d1, d2) for s in sources):
            total += w
    return total


def my_blots(state: State) -> list[int]:
    """My points holding exactly one free (not pinned, not pinning) checker."""
    return [
        p
        for p in range(1, 25)
        if state.me[p] == 1 and not state.i_pin_at(p) and not state.my_pinned_at(p)
    ]


def blot_risks(state: State) -> dict[int, int]:
    """For each of my blots: rolls out of 36 with which the opponent could pin it
    on their next turn. Call it on the position *after* my move (mover's view)."""
    opp_view = state.flip()
    return {p: pin_shots(opp_view, 25 - p) for p in my_blots(state)}


def mother_exposed(state: State) -> bool:
    """True if exactly one of my checkers is left on my start (it can be pinned)."""
    return state.me[START] == 1 and not state.my_pinned_at(START)
