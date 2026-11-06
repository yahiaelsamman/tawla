"""Cross-check the optimised move generator against a naive brute-force one."""
import random

from mahbusa import EGYPTIAN, State, legal_moves
from mahbusa.agents import RandomAgent
from mahbusa.game import play_game
from mahbusa.state import _step


def naive_moves(state: State, d1: int, d2: int, rules=EGYPTIAN):
    orders = [(d1,) * 4] if d1 == d2 else [(d1, d2), (d2, d1)]
    n = len(orders[0])
    leaves = []  # (used, first_die, key, outcome)

    def rec(me, mpin, dice, idx, steps):
        any_move = False
        if idx < len(dice):
            for p in range(1, 25):
                r = _step(me, state.op, mpin, state.opin, p, dice[idx], rules)
                if r is None:
                    continue
                any_move = True
                nme, _, nmpin, _, to, outcome = r
                if outcome is not None:
                    leaves.append((n, dice[0], (tuple(nme), nmpin), outcome))
                else:
                    rec(nme, nmpin, dice, idx + 1, steps + ((p, to),))
        if not any_move:
            leaves.append((idx, dice[0] if idx else None, (tuple(me), mpin), None))

    for dice in orders:
        rec(list(state.me), state.mpin, dice, 0, ())
    best = max(l[0] for l in leaves)
    if best == 0:
        return set()
    chosen = [l for l in leaves if l[0] == best]
    if best == 1 and d1 != d2 and any(l[1] == max(d1, d2) for l in chosen):
        chosen = [l for l in chosen if l[1] == max(d1, d2)]
    return {(l[2], l[3]) for l in chosen}


def collect_positions(n_games=40, seed=11):
    rng = random.Random(seed)
    positions = []
    for _ in range(n_games):
        r = play_game([RandomAgent(), RandomAgent()], rng, record=True)
        state = State.initial(turn=r.starter)
        for player, dice, move in r.history:
            positions.append(state)
            state = state.flip() if move is None else move.after.flip()
    return positions


def test_fast_generator_matches_bruteforce():
    rng = random.Random(5)
    positions = collect_positions()
    assert len(positions) > 1000
    for s in positions:
        d1, d2 = rng.randint(1, 6), rng.randint(1, 6)
        fast = {((m.after.me, m.after.mpin), m.outcome) for m in legal_moves(s, d1, d2)}
        assert fast == naive_moves(s, d1, d2), (s, d1, d2)
