"""Rule tests for the Mahbusa engine. Positions are built from the mover's view."""
import random

import pytest

from mahbusa import EGYPTIAN, PLAKOTO, State, legal_moves, play_game
from mahbusa.agents import HeuristicAgent, RandomAgent
from mahbusa.features import pin_shots
from shot_table import shots as table_shots


def make(me: dict, op: dict, me_off=0, op_off=0, mpin=(), opin=(), turn=0) -> State:
    """me/op: {point: count} in each side's OWN numbering. mpin: my points where I pin;
    opin: opponent's points (their numbering) where they pin me."""
    m = [0] * 25
    o = [0] * 25
    for p, c in me.items():
        m[p] += c
    for p, c in op.items():
        o[p] += c
    m[0] += me_off
    o[0] += op_off
    s = State(m, o, sum(1 << p for p in mpin), sum(1 << q for q in opin), turn)
    s.validate()
    return s


def results(s, d1, d2, rules=EGYPTIAN):
    return {m.steps: m for m in legal_moves(s, d1, d2, rules)}


def afters(s, d1, d2, rules=EGYPTIAN):
    return [m.after for m in legal_moves(s, d1, d2, rules)]


# ---------------------------------------------------------------- setup
def test_initial_position():
    s = State.initial()
    s.validate()
    assert s.me[24] == 15 and s.op[24] == 15
    assert s.opp_at(1) == 15  # opponent's stack sits on my 1-point
    assert s.pip_count() == (360, 360)


def test_flip_is_involution():
    s = make({24: 10, 18: 3, 5: 2}, {24: 13, 19: 1, 3: 1})
    assert s.flip().flip() == s


@pytest.mark.parametrize("roll,count", [((2, 1), 2), ((6, 6), 4), ((5, 5), 5), ((1, 1), 5), ((6, 5), 2)])
def test_opening_move_counts(roll, count):
    assert len(legal_moves(State.initial(), *roll)) == count


# ---------------------------------------------------------------- pinning
def test_landing_on_blot_pins_it():
    # Opponent blot on their 20 = my 5. I move 9 -> 5 with a 4.
    s = make({24: 13, 9: 2}, {24: 14, 20: 1})
    after = [m.after for m in legal_moves(s, 4, 1) if (9, 5) in m.steps]
    assert after, "pinning move should be legal"
    a = after[0]
    assert a.i_pin_at(5)
    a.validate()


def test_pinned_checker_cannot_move():
    # I pin the opponent's checker on my 5 (their 20). From their side, that checker is frozen.
    s = make({24: 13, 5: 1, 9: 1}, {24: 14, 20: 1}, mpin=[5])
    opp = s.flip()
    assert opp.my_pinned_at(20)
    for m in legal_moves(opp, 3, 2):
        assert all(src != 20 for src, _ in m.steps)


def test_pin_point_is_blocked_for_pinned_player_but_open_for_pinner():
    s = make({24: 12, 5: 1, 7: 1, 10: 1}, {24: 13, 20: 1, 22: 1}, mpin=[5])
    # Pinner side can pile on: 7 -> 5 with a 2.
    assert any((7, 5) in m.steps for m in legal_moves(s, 2, 1))
    # Pinned side cannot land on their 20 (my 5): their 22 -> 20 with a 2 is illegal.
    opp = s.flip()
    assert not any((22, 20) in m.steps for m in legal_moves(opp, 2, 1))


def test_two_checkers_block():
    s = make({24: 14, 9: 1}, {24: 13, 20: 2})  # opponent block on my 5
    assert not any((9, 5) in m.steps for m in legal_moves(s, 4, 6))


def test_release_when_last_pinner_leaves():
    s = make({24: 13, 5: 1, 9: 1}, {24: 14, 20: 1}, mpin=[5])
    moved = [m.after for m in legal_moves(s, 3, 1) if any(src == 5 for src, _ in m.steps)]
    assert moved
    for a in moved:
        assert not a.i_pin_at(5)
        a.validate()
    # The released checker can move again.
    assert any(any(src == 20 for src, _ in m.steps) for m in legal_moves(moved[0].flip(), 1, 2))


def test_cannot_pin_a_pinner():
    # Opponent pins my checker on their 20 (my 5) with a single checker: I cannot land there.
    s = make({24: 13, 5: 1, 8: 1}, {24: 14, 20: 1}, opin=[20])
    assert not any((8, 5) in m.steps for m in legal_moves(s, 3, 1))


def test_combined_move_needs_open_intermediate_point():
    # Single checker 10 -> 4 with 3-3 would need 7 open; opponent block on my 7 (their 18).
    s = make({24: 14, 10: 1}, {24: 13, 18: 2})
    for m in legal_moves(s, 3, 3):
        assert (10, 7) not in m.steps


# ---------------------------------------------------------------- mother
def test_mother_pin_wins_double_immediately():
    # Opponent has exactly one checker left on their start (my 1-point). I land there.
    s = make({24: 10, 4: 5}, {24: 1, 13: 14})
    wins = [m for m in legal_moves(s, 3, 6) if m.outcome is not None]
    assert wins and all(m.outcome == 2 for m in wins)


def test_mother_exception_plakoto():
    # Greek rule: I still have checkers on my own start, so no immediate win.
    s = make({24: 10, 4: 5}, {24: 1, 13: 14})
    pin_moves = [m for m in legal_moves(s, 3, 6, PLAKOTO) if m.after.i_pin_at(1)]
    assert pin_moves and all(m.outcome is None for m in pin_moves)


def test_game_ends_with_mother_reason():
    rng = random.Random(3)
    r = play_game([RandomAgent(), RandomAgent()], rng)
    assert r.reason in {"bearoff", "mars", "mother"}


# ---------------------------------------------------------------- bearing off
def test_no_bearoff_with_checker_outside_home():
    s = make({7: 1, 3: 14}, {24: 15})
    for m in legal_moves(s, 3, 3):
        # the 7 checker must enter first; afterwards bearing off is allowed
        assert m.steps[0][0] == 7


def test_bear_off_exact_and_from_highest():
    s = make({3: 2, 2: 3}, {24: 15}, me_off=10)
    steps = {m.steps for m in legal_moves(s, 6, 5)}
    # With 6 and 5 and nothing above 3: bear off from the highest point (3) twice.
    assert ((3, 0), (3, 0)) in steps


def test_higher_die_cannot_bear_off_lower_when_higher_points_occupied():
    # A 5 cannot bear off from the 2-point while a checker sits on the 6-point.
    s = make({6: 1, 2: 1}, {24: 15}, me_off=13)
    for m in legal_moves(s, 5, 1):
        assert (2, 0) not in m.steps


def test_no_bearoff_while_pinned_in_home():
    # My checker on my 3 is pinned by the opponent (their 22). Everything else is home.
    s = make({6: 4, 3: 1, 2: 10}, {22: 1, 10: 14}, opin=[22])
    for m in legal_moves(s, 6, 2):
        assert all(to != 0 for _, to in m.steps)


def test_bearing_off_is_optional():
    s = make({6: 1, 1: 1}, {12: 15}, me_off=13)
    steps = {m.steps for m in legal_moves(s, 1, 2)}
    assert any(all(t != 0 for _, t in st) for st in steps)  # e.g. 6/4 4/3
    assert any(any(t == 0 for _, t in st) for st in steps)  # e.g. 6/4 1/off


def test_single_and_double_win():
    s = make({1: 1}, {12: 15}, me_off=14)
    assert legal_moves(s, 1, 2)[0].outcome == 2  # opponent has nothing off: mars
    s = make({1: 1}, {12: 14}, me_off=14, op_off=1)
    assert legal_moves(s, 1, 2)[0].outcome == 1


# ---------------------------------------------------------------- forced moves
def test_must_use_both_dice_when_possible():
    s = make({24: 14, 10: 1}, {24: 13, 21: 2})  # opponent block on my 4-point
    ms = legal_moves(s, 6, 1)
    assert ms and all(len(m.steps) == 2 for m in ms)


def test_must_play_larger_die_if_only_one_can_be_played():
    # Only my 8-point checker can move. It can play the 5 (8/3) or the 2 (8/6),
    # but never both: 3/1 and 6/1 land on the opponent's start stack.
    # My 20-point checker is walled in (my 15 and 18 blocked); not all home, so no bear-off.
    s = make({8: 1, 20: 1, 2: 13}, {24: 2, 10: 2, 7: 2, 13: 9})
    ms = legal_moves(s, 5, 2)
    assert [m.steps for m in ms] == [((8, 3),)]


def test_no_legal_move_means_pass():
    # A 6-point wall right in front of my start (my 18..23 = their 7..2).
    s = make({24: 15}, {7: 2, 6: 2, 5: 2, 4: 2, 3: 2, 2: 2, 24: 3})
    assert legal_moves(s, 6, 6) == []


# ---------------------------------------------------------------- features
@pytest.mark.parametrize("dist", range(1, 25))
def test_pin_shots_match_table_on_open_board(dist):
    # One of my checkers at 24 -> target dist below, no blocks in the way.
    target = 24 - dist
    if target < 1:
        return
    s = make({24: 15}, {25 - target: 1}, op_off=14)
    assert pin_shots(s, target) == table_shots(dist)


# ---------------------------------------------------------------- whole games
@pytest.mark.parametrize("seed", range(20))
def test_random_games_keep_invariants(seed):
    rng = random.Random(seed)
    r = play_game([RandomAgent(), RandomAgent()], rng, record=True)
    assert r.points in (1, 2)
    for player, dice, move in r.history:
        if move is not None:
            move.after.validate()


def test_heuristic_beats_random():
    from mahbusa.game import play_many

    t = play_many([HeuristicAgent(), RandomAgent()], 60, seed=7)
    assert t.wins[0] > 40
