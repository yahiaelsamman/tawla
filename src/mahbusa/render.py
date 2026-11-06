"""Text rendering of a position, always drawn from White's (player 0) side."""
from __future__ import annotations

from .state import State

SYMBOL = {0: "X", 1: "O"}


def _absolute(state: State):
    """Return (white[25], black_in_white_numbering[25], white_pins, black_pins),
    pins as sets of White-numbered points."""
    if state.turn == 0:
        white, black, wpin_mask, bpin_mask = state.me, state.op, state.mpin, state.opin
    else:
        white, black, wpin_mask, bpin_mask = state.op, state.me, state.opin, state.mpin
    black_w = [0] * 25
    for q in range(1, 25):
        black_w[25 - q] = black[q]
    wpins = {p for p in range(1, 25) if (wpin_mask >> p) & 1}
    bpins = {25 - q for q in range(1, 25) if (bpin_mask >> q) & 1}
    return white, black, black_w, wpins, bpins


def _cell(w: int, b: int, wpin: bool, bpin: bool) -> str:
    if wpin:
        return f"{w}X/o"  # w White checkers pinning one Black checker
    if bpin:
        return f"{b}O/x"
    if w:
        return f"{w}X"
    if b:
        return f"{b}O"
    return "."


def render(state: State) -> str:
    white, black, black_w, wpins, bpins = _absolute(state)
    cells = {p: _cell(white[p], black_w[p], p in wpins, p in bpins) for p in range(1, 25)}

    def row(points):
        left = " ".join(f"{cells[p]:>5}" for p in points[:6])
        right = " ".join(f"{cells[p]:>5}" for p in points[6:])
        return f"{left} |{right}"

    def nums(points):
        left = " ".join(f"{p:>5}" for p in points[:6])
        right = " ".join(f"{p:>5}" for p in points[6:])
        return f"{left} |{right}"

    top = list(range(13, 25))
    bottom = list(range(12, 0, -1))
    wp = sum(p * c for p, c in enumerate(white))
    bp = sum(q * c for q, c in enumerate(black))
    lines = [
        "Numbers are White's (X) points. X moves 24 -> 1; O moves 1 -> 24.",
        nums(top),
        row(top),
        row(bottom),
        nums(bottom),
        f"X off: {white[0]:2d}  pips: {wp:3d}    O off: {black[0]:2d}  pips: {bp:3d}    "
        f"to move: {SYMBOL[state.turn]}",
        "(nX/o = n X checkers pinning one O checker; nO/x likewise)",
    ]
    return "\n".join(lines)
