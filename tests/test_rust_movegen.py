"""The Rust engine (engine-rs) must generate exactly the same moves as the Python one."""
import random
import subprocess
from pathlib import Path

import pytest

from mahbusa import legal_moves
from test_movegen_bruteforce import collect_positions

BIN = Path(__file__).resolve().parent.parent / "engine-rs" / "target" / "release" / "mahbusa-rs"


@pytest.mark.skipif(not BIN.exists(), reason="build engine-rs first: cargo build --release")
def test_rust_matches_python():
    rng = random.Random(21)
    positions = collect_positions(n_games=150, seed=4)
    cases = [(s, rng.randint(1, 6), rng.randint(1, 6)) for s in positions]
    lines = [
        f"{','.join(map(str, s.me))};{','.join(map(str, s.op))};{s.mpin};{s.opin};{d1};{d2}"
        for s, d1, d2 in cases
    ]
    out = subprocess.run([str(BIN), "movegen"], input="\n".join(lines) + "\n",
                         capture_output=True, text=True, check=True).stdout.splitlines()
    assert len(out) == len(cases)
    for (s, d1, d2), line in zip(cases, out):
        py = sorted(f"{','.join(map(str, m.after.me))}|{m.after.mpin}|{m.outcome or 0}"
                    for m in legal_moves(s, d1, d2))
        rs = sorted(line.split()) if line.strip() else []
        assert py == rs, (s, d1, d2)
