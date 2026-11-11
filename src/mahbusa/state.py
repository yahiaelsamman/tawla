"""Board state and legal-move generation.

Conventions
-----------
The state is always stored from the point of view of the player to move ("me"):

* ``me[1..24]``: my checkers on each point, in MY numbering. I start on 24, move down,
  and my home is 1..6. ``me[0]`` is the number of my checkers borne off.
* ``op[1..24]``: the opponent's checkers in THEIR OWN numbering (same convention).
  My point ``p`` is the opponent's point ``25 - p``. ``op[0]`` is their borne-off count.
* ``mpin``: bitmask over MY points. Bit ``p`` is set when my checker(s) on point ``p``
  are pinning a lone opponent checker.
* ``opin``: bitmask over the OPPONENT's points. Bit ``q`` is set when they pin one of
  my checkers on their point ``q`` (my point ``25 - q``).
* ``turn``: 0 or 1, the absolute identity of the player to move (0 = White, 1 = Black).
  It is only used for display and bookkeeping.

``flip()`` swaps the two sides, so after a move the state is flipped and handed to the
opponent. Because the representation is symmetric, flipping is just swapping fields.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import NamedTuple

from .rules import EGYPTIAN, Rules

N_CHECKERS = 15
START = 24


class State:
    __slots__ = ("me", "op", "mpin", "opin", "turn")

    def __init__(self, me, op, mpin=0, opin=0, turn=0):
        self.me = tuple(me)
        self.op = tuple(op)
        self.mpin = mpin
        self.opin = opin
        self.turn = turn

    # ------------------------------------------------------------------ basics
    @classmethod
    def initial(cls, turn: int = 0) -> "State":
        side = [0] * 25
        side[START] = N_CHECKERS
        return cls(side, side, 0, 0, turn)

    def flip(self) -> "State":
        return State(self.op, self.me, self.opin, self.mpin, 1 - self.turn)

    def key(self):
        return (self.me, self.op, self.mpin, self.opin)

    def __eq__(self, other):
        return isinstance(other, State) and self.key() == other.key() and self.turn == other.turn

    def __hash__(self):
        return hash((self.key(), self.turn))

    def __repr__(self):
        return f"State(me={list(self.me)}, op={list(self.op)}, mpin={self.mpin:#x}, opin={self.opin:#x}, turn={self.turn})"

    # ------------------------------------------------------------ queries
    def opp_at(self, p: int) -> int:
        """Number of opponent checkers on MY point p (1..24)."""
        return self.op[25 - p]

    def my_pinned_at(self, p: int) -> bool:
        """Is my checker on my point p pinned by the opponent?"""
        return bool((self.opin >> (25 - p)) & 1)

    def i_pin_at(self, p: int) -> bool:
        """Do I pin an opponent checker on my point p?"""
        return bool((self.mpin >> p) & 1)

    def pip_count(self) -> tuple[int, int]:
        """(my pips, opponent pips) still to travel before everything is borne off."""
        return (sum(p * c for p, c in enumerate(self.me)), sum(p * c for p, c in enumerate(self.op)))

    def validate(self) -> None:
        """Raise AssertionError if the state breaks an invariant (used by tests)."""
        assert len(self.me) == 25 and len(self.op) == 25
        assert sum(self.me) == N_CHECKERS, self
        assert sum(self.op) == N_CHECKERS, self
        assert all(c >= 0 for c in self.me + self.op)
        for p in range(1, 25):
            mine, theirs = self.me[p], self.opp_at(p)
            i_pin, they_pin = self.i_pin_at(p), self.my_pinned_at(p)
            assert not (i_pin and they_pin), (p, self)
            if mine and theirs:
                # Both colours on one point is only possible with a pin.
                assert i_pin or they_pin, (p, self)
            if i_pin:
                assert theirs == 1 and mine >= 1, (p, self)
            if they_pin:
                assert mine == 1 and theirs >= 1, (p, self)
        assert self.mpin & ~((1 << 25) - 2) == 0
        assert self.opin & ~((1 << 25) - 2) == 0


class Move(NamedTuple):
    """A full move for one roll.

    steps:   ((from, to), ...) in the mover's numbering; to == 0 means borne off.
    after:   the resulting State, still from the MOVER's point of view (not flipped).
    outcome: None if the game goes on; otherwise the mover's result in points
             (+1 single win, +2 double win / mother pin, 0 draw).
    """

    steps: tuple
    after: State
    outcome: int | None


# ---------------------------------------------------------------------- helpers
def _can_land(me, op, opin, q) -> bool:
    """Can I land on my point q (1..24)?"""
    oc = op[25 - q]
    if oc == 0:
        return True
    if oc == 1:
        # A lone opponent checker can be pinned (or piled on if I already pin it),
        # unless it is their pinner sitting on my checker.
        return not ((opin >> (25 - q)) & 1)
    return False


def _all_home(me) -> bool:
    for p in range(7, 25):
        if me[p]:
            return False
    return True


def _highest(me) -> int:
    for p in range(24, 0, -1):
        if me[p]:
            return p
    return 0


def _step(me, op, mpin, opin, p, d, rules: Rules):
    """Try to move one of my checkers from p by d pips.

    Returns None if illegal, else (me, op, mpin, opin, to, outcome).
    me/op are lists (copies); outcome is None or the mover's points.
    """
    if me[p] == 0:
        return None
    if (opin >> (25 - p)) & 1:  # my checker here is pinned
        return None
    q = p - d
    if q >= 1:
        if not _can_land(me, op, opin, q):
            return None
    else:
        # bearing off
        if opin or not _all_home(me):
            return None
        if q < 0 and _highest(me) != p:
            return None
    me = list(me)
    me[p] -= 1
    if me[p] == 0 and (mpin >> p) & 1:
        mpin &= ~(1 << p)  # last pinner left: opponent checker released
    outcome = None
    if q >= 1:
        me[q] += 1
        if op[25 - q] == 1 and not (mpin >> q) & 1:
            mpin |= 1 << q  # new pin
            if q == 1:  # opponent's starting point: their last checker = mother
                outcome = _mother_outcome(me, op, opin, rules)
        to = q
    else:
        me[0] += 1
        to = 0
        if me[0] == N_CHECKERS:
            outcome = 2 if op[0] == 0 else 1
    # Greek rule only: a mother pin that was deferred because my own start still held
    # checkers becomes a win as soon as my start is empty.
    if outcome is None and rules.mother_exception and (mpin >> 1) & 1 and me[START] == 0:
        outcome = rules.mother_points
    return me, op, mpin, opin, to, outcome


def _mother_outcome(me, op, opin, rules: Rules):
    if not rules.mother_exception:
        return rules.mother_points
    # Greek Plakoto: no immediate win if my own start still holds checkers.
    if me[START] > 0:
        if me[START] == 1 and (opin >> 1) & 1:
            return 0  # both mothers pinned: draw
        return None
    return rules.mother_points


# ------------------------------------------------------------- move generation
def legal_moves(state: State, d1: int, d2: int, rules: Rules = EGYPTIAN) -> list[Move]:
    """All distinct legal moves for the roll (d1, d2).

    Enforces: use as many dice as possible; if only one die of a non-double can be
    used, use the larger when possible. A sequence that ends the game early (mother
    pin, last checker off) counts as using all dice. Returns [] if no move is possible
    (the player passes).
    """
    orders = [(d1,) * 4] if d1 == d2 else [(d1, d2), (d2, d1)]
    n_dice = len(orders[0])
    leaves: dict = {}  # result key -> (used, first die used, steps, me, op, mpin, opin, outcome)
    seen = set()

    def rec(me, op, mpin, opin, dice, idx, steps):
        sk = (tuple(me), mpin, opin, dice[idx:] if idx < len(dice) else ())
        if sk in seen:
            return
        seen.add(sk)
        moved = False
        if idx < len(dice):
            d = dice[idx]
            for p in range(24, 0, -1):
                if me[p] == 0:
                    continue
                r = _step(me, op, mpin, opin, p, d, rules)
                if r is None:
                    continue
                moved = True
                nme, nop, nmpin, nopin, to, outcome = r
                nsteps = steps + ((p, to),)
                if outcome is not None:
                    _record(nme, nop, nmpin, nopin, n_dice, nsteps, outcome, dice[0])
                else:
                    rec(nme, nop, nmpin, nopin, dice, idx + 1, nsteps)
        if not moved:
            _record(me, op, mpin, opin, idx, steps, None, dice[0] if idx else None)

    def _record(me, op, mpin, opin, used, steps, outcome, first_die):
        k = (tuple(me), mpin, opin)
        prev = leaves.get(k)
        # Prefer the record that used more dice; on a tie prefer the larger first
        # die, so the "play the larger die" filter below sees it.
        if prev is None or (used, first_die or 0) > (prev[0], prev[1] or 0):
            leaves[k] = (used, first_die, steps, me, op, mpin, opin, outcome)

    for dice in orders:
        rec(list(state.me), state.op, state.mpin, state.opin, dice, 0, ())

    if not leaves:
        return []
    max_used = max(v[0] for v in leaves.values())
    if max_used == 0:
        return []
    chosen = [v for v in leaves.values() if v[0] == max_used]
    if max_used == 1 and d1 != d2:
        big = max(d1, d2)
        with_big = [v for v in chosen if v[1] == big]
        if with_big:
            chosen = with_big
    return [
        Move(v[2], State(v[3], v[4], v[5], v[6], state.turn), v[7])
        for v in chosen
    ]


def format_move(move_or_steps) -> str:
    steps = move_or_steps.steps if isinstance(move_or_steps, Move) else move_or_steps
    if not steps:
        return "(no move)"
    return " ".join(f"{a}/{'off' if b == 0 else b}" for a, b in steps)
