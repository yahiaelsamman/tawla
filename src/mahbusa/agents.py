"""Simple players. An agent picks one move from the list of legal moves.

These are baselines for testing the engine and for benchmarking the neural network
later. They are deliberately simple.
"""
from __future__ import annotations

import random

from .features import blot_risks, mother_exposed, pin_shots
from .state import START, Move, State


class Agent:
    name = "agent"

    def choose(self, state: State, moves: list[Move], dice: tuple[int, int], rng: random.Random) -> Move:
        raise NotImplementedError


class RandomAgent(Agent):
    name = "random"

    def choose(self, state, moves, dice, rng):
        return rng.choice(moves)


class HeuristicAgent(Agent):
    """One-ply greedy player with hand-set weights, based on the v1 strategy guide:
    win immediately if possible, value pins (more the deeper in my home), value
    blocks, avoid pinnable blots and an exposed mother, prefer progress."""

    name = "heuristic"

    def __init__(self, weights: dict | None = None):
        self.w = {
            "pip": 0.01,          # per pip of race lead
            "pin": 1.0,           # base value of a pin
            "pin_depth": 0.25,    # extra per point closer to the opponent's start (my 1-point)
            "block": 0.25,        # per made point (2+ of my checkers)
            "blot_risk": 2.0,     # times probability my blot gets pinned
            "mother": 3.0,        # penalty for a lone, pinnable mother
            "pinned": 1.5,        # penalty per checker of mine that is pinned
        }
        if weights:
            self.w.update(weights)

    def evaluate(self, s: State) -> float:
        w = self.w
        my_pips, op_pips = s.pip_count()
        score = w["pip"] * (op_pips - my_pips)
        for p in range(1, 25):
            if s.i_pin_at(p):
                score += w["pin"] + w["pin_depth"] * max(0, 12 - p)
            elif s.me[p] >= 2:
                score += w["block"]
            if s.my_pinned_at(p):
                score -= w["pinned"] + w["pin_depth"] * max(0, p - 13)
        for p, shots in blot_risks(s).items():
            danger = 1.0 + max(0, p - 12) * 0.25  # a pin far back costs more
            score -= w["blot_risk"] * danger * shots / 36
        if mother_exposed(s):
            shots = pin_shots(s.flip(), 25 - START)
            score -= w["mother"] * shots / 36 * 10
        return score

    def choose(self, state, moves, dice, rng):
        best, best_val = None, None
        for m in moves:
            if m.outcome is not None:
                val = 1000.0 * m.outcome
            else:
                val = self.evaluate(m.after)
            if best_val is None or val > best_val:
                best, best_val = m, val
        return best

