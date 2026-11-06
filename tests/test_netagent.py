import random

import pytest

from mahbusa.agents import RandomAgent
from mahbusa.game import play_game

netagent = pytest.importorskip("mahbusa.netagent")


def _available():
    try:
        netagent.find_binary()
        netagent.find_net()
        return True
    except FileNotFoundError:
        return False


@pytest.mark.skipif(not _available(), reason="needs the Rust engine and a trained net")
def test_net_agent_plays_full_games():
    agent = netagent.NetAgent()
    try:
        for seed in range(3):
            r = play_game([agent, RandomAgent()], random.Random(seed))
            assert r.points in (1, 2)
    finally:
        agent.close()
