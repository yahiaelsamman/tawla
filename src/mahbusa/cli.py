"""Command line: play against a bot, or simulate games between bots.

    mahbusa play [--bot heuristic|random] [--plakoto]
    mahbusa sim  [--a heuristic] [--b random] [-n 1000] [--seed 0] [--plakoto]
"""
from __future__ import annotations

import argparse
import random
import time

from .agents import Agent, HeuristicAgent, RandomAgent
from .game import play_game, play_many
from .render import render
from .rules import EGYPTIAN, PLAKOTO
from .state import format_move

BOTS = {"random": RandomAgent, "heuristic": HeuristicAgent}


def _net_agent():
    from .netagent import NetAgent

    return NetAgent()


BOTS["net"] = _net_agent


class HumanAgent(Agent):
    name = "you"

    def choose(self, state, moves, dice, rng):
        print()
        print(render(state))
        print(f"\nYou rolled {dice[0]}-{dice[1]}. Your points are numbered from YOUR side (start = 24).")
        for i, m in enumerate(moves):
            tag = "  <- wins the game" if m.outcome else ""
            print(f"  {i:2d}: {format_move(m)}{tag}")
        while True:
            raw = input("Choose a move number: ").strip()
            if raw.isdigit() and int(raw) < len(moves):
                return moves[int(raw)]
            print("Please type one of the numbers above.")


class Narrator(Agent):
    """Wraps a bot and prints what it plays."""

    def __init__(self, inner: Agent):
        self.inner = inner
        self.name = inner.name

    def choose(self, state, moves, dice, rng):
        m = self.inner.choose(state, moves, dice, rng)
        eq = getattr(self.inner, "last_equity", None)
        extra = f"  [bot's estimate: {eq:+.2f} points for itself]" if eq is not None else ""
        mine = " ".join(f"{25 - x}/{'off' if y == 0 else 25 - y}" for x, y in m.steps) or "(no move)"
        print(f"\nBot rolled {dice[0]}-{dice[1]} and played {mine} (in YOUR point numbers).{extra}")
        return m


def cmd_play(args):
    rules = PLAKOTO if args.plakoto else EGYPTIAN
    rng = random.Random(args.seed)
    agents = [HumanAgent(), Narrator(BOTS[args.bot]())]
    r = play_game(agents, rng, rules)
    who = {0: "You win", 1: "The bot wins", None: "Draw"}[r.winner]
    print(f"\n{who}: {r.points} point(s) by {r.reason} after {r.turns} turns.")


def cmd_sim(args):
    rules = PLAKOTO if args.plakoto else EGYPTIAN
    a, b = BOTS[args.a](), BOTS[args.b]()
    t0 = time.time()
    tally = play_many([a, b], args.n, seed=args.seed, rules=rules)
    dt = time.time() - t0
    print(tally.summary((args.a, args.b)))
    print(f"{args.n / dt:.0f} games/second")


def main(argv=None):
    ap = argparse.ArgumentParser(prog="mahbusa", description="Mahbusa (Egyptian Plakoto) engine")
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("play", help="play against a bot in the terminal")
    p.add_argument("--bot", choices=BOTS, default="net")
    p.add_argument("--seed", type=int, default=None)
    p.add_argument("--plakoto", action="store_true", help="Greek mother-checker exception")
    p.set_defaults(func=cmd_play)
    s = sub.add_parser("sim", help="simulate games between two bots")
    s.add_argument("--a", choices=BOTS, default="heuristic")
    s.add_argument("--b", choices=BOTS, default="random")
    s.add_argument("-n", type=int, default=1000)
    s.add_argument("--seed", type=int, default=0)
    s.add_argument("--plakoto", action="store_true")
    s.set_defaults(func=cmd_sim)
    args = ap.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
