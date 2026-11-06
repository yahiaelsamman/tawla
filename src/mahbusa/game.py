"""Playing games and sets between agents."""
from __future__ import annotations

import random
from dataclasses import dataclass, field

from .rules import EGYPTIAN, Rules
from .state import Move, State, legal_moves

MAX_TURNS = 2000  # safety net; real games last well under 200 turns


@dataclass
class GameResult:
    winner: int | None      # 0 or 1 (absolute player), None for a draw
    points: int             # points won by the winner (1 or 2; 0 for a draw)
    reason: str             # "bearoff", "mars", "mother" or "draw"
    turns: int
    starter: int
    history: list = field(default_factory=list)  # [(player, dice, Move or None)]


def roll(rng: random.Random) -> tuple[int, int]:
    return rng.randint(1, 6), rng.randint(1, 6)


def opening_starter(rng: random.Random) -> int:
    """Each player rolls one die; the higher one starts (ties re-roll). The starter
    then rolls both dice for the first move, as in Plakoto."""
    while True:
        a, b = rng.randint(1, 6), rng.randint(1, 6)
        if a != b:
            return 0 if a > b else 1


def play_game(
    agents,
    rng: random.Random | None = None,
    rules: Rules = EGYPTIAN,
    starter: int | None = None,
    record: bool = False,
    start_state: State | None = None,
) -> GameResult:
    """Play one game. agents[0] is White (player 0), agents[1] is Black."""
    rng = rng or random.Random()
    if starter is None:
        starter = opening_starter(rng)
    state = start_state if start_state is not None else State.initial(turn=starter)
    history = []
    for turn in range(1, MAX_TURNS + 1):
        player = state.turn
        dice = roll(rng)
        moves = legal_moves(state, *dice, rules)
        if not moves:
            if record:
                history.append((player, dice, None))
            state = state.flip()
            continue
        move = moves[0] if len(moves) == 1 else agents[player].choose(state, moves, dice, rng)
        if record:
            history.append((player, dice, move))
        if move.outcome is not None:
            return _finish(move, player, turn, starter, history, state, move.steps)
        state = move.after.flip()
    raise RuntimeError("game did not finish")


def _finish(move: Move, player, turn, starter, history, state, steps) -> GameResult:
    pts = move.outcome
    if pts == 0:
        return GameResult(None, 0, "draw", turn, starter, history)
    if move.after.me[0] == 15:
        reason = "mars" if pts == 2 else "bearoff"
    else:
        reason = "mother"
    return GameResult(player, pts, reason, turn, starter, history)


@dataclass
class Tally:
    games: int = 0
    wins: list = field(default_factory=lambda: [0, 0])
    points: list = field(default_factory=lambda: [0, 0])
    reasons: dict = field(default_factory=dict)
    draws: int = 0
    turns: int = 0

    def add(self, r: GameResult):
        self.games += 1
        self.turns += r.turns
        self.reasons[r.reason] = self.reasons.get(r.reason, 0) + 1
        if r.winner is None:
            self.draws += 1
        else:
            self.wins[r.winner] += 1
            self.points[r.winner] += r.points

    def summary(self, names=("player 0", "player 1")) -> str:
        n = max(self.games, 1)
        ppg = (self.points[0] - self.points[1]) / n
        win_pct = 100 * self.wins[0] / n
        se = 100 * (win_pct / 100 * (1 - win_pct / 100) / n) ** 0.5
        lines = [
            f"{self.games} games: {names[0]} {self.wins[0]} wins, {names[1]} {self.wins[1]} wins, {self.draws} draws",
            f"{names[0]} win rate {win_pct:.1f}% ± {1.96 * se:.1f}% (95% CI), points/game {ppg:+.3f}",
            "endings: " + ", ".join(f"{k} {100 * v / n:.1f}%" for k, v in sorted(self.reasons.items())),
            f"average game length: {self.turns / n:.1f} turns",
        ]
        return "\n".join(lines)


def play_many(agents, n: int, seed: int = 0, rules: Rules = EGYPTIAN) -> Tally:
    """Play n independent games, alternating who starts, from agents[0]'s side."""
    rng = random.Random(seed)
    tally = Tally()
    for i in range(n):
        tally.add(play_game(agents, rng, rules, starter=i % 2))
    return tally
