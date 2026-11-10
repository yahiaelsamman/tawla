"""Write test cases for the browser engine: 'me;op;mpin;opin;d1;d2' lines from random games.

    python web/tests/make_cases.py N_GAMES OUT_FILE
"""
import random
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tests"))
sys.path.insert(0, str(ROOT / "src"))
from test_movegen_bruteforce import collect_positions  # noqa: E402

n_games, out = int(sys.argv[1]), sys.argv[2]
rng = random.Random(2026)
lines = []
for s in collect_positions(n_games=n_games, seed=17):
    d1, d2 = rng.randint(1, 6), rng.randint(1, 6)
    lines.append(f"{','.join(map(str, s.me))};{','.join(map(str, s.op))};{s.mpin};{s.opin};{d1};{d2}")
Path(out).write_text("\n".join(lines) + "\n")
print(f"{len(lines)} cases")
