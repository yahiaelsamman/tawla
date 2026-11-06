# Mahbusa (محبوسة) strategy project

The goal is to work out and write down winning strategy for Mahbusa, the Egyptian pinning variant of backgammon. Internationally it's known as Plakoto (Greece) or Tapa (Bulgaria).

| Path | What it is |
|---|---|
| `guide/STRATEGY_GUIDE.md` | **The strategy guide.** Every claim is tagged with how solid it is. |
| `web/` | **Play in the browser**: open `web/index.html` (no install). A wooden tawla board, the trained bot, and a coach that rates your moves. |
| `research/RESEARCH.md` | Rules, sources, the existing strategy knowledge and computer analysis. |
| `src/mahbusa/` | The game engine (Python, no dependencies). |
| `tests/` | Rule tests plus a brute-force cross-check of the move generator. |
| `scripts/shot_table.py` | Pin-shot odds used in the guide. |
| `research/sources/` | Local copies of source pages and papers (git-ignored). |

## Rules implemented (Egyptian Mahbusa)

All 15 checkers start on your 24-point (the opponent's 1-point). Players move in opposite directions. Landing on a lone enemy checker **pins** it until all your checkers leave that point. A pin or 2+ checkers make a block. Pinning the opponent's last checker on their start (the **mother**) wins at once, double. Bear off only when all 15 are home and none is pinned. A win is 1 point, or 2 (**mars**) if the loser has borne off nothing. Full rules are in `research/RESEARCH.md` §2. The Greek mother exception is available with `--plakoto` or `Rules(mother_exception=True)`.

## Usage

```bash
python3 -m venv .venv
.venv/bin/pip install -e ".[dev]"
```

Run the tests:

```bash
.venv/bin/pytest
```

Play against the heuristic bot in the terminal:

```bash
.venv/bin/mahbusa play
```

Bot-vs-bot simulation:

```bash
.venv/bin/mahbusa sim --a heuristic --b random -n 1000
```

In Python:

```python
from mahbusa import State, legal_moves, format_move
from mahbusa.render import render

s = State.initial()
print(render(s))
for m in legal_moves(s, 6, 4):
    print(format_move(m))
```

### Engine conventions

- The state is always seen from the **player to move**: `me[p]` / `op[q]` are each side's checkers in their own numbering (start = 24, home = 1–6, index 0 = borne off). The opponent's point `q` is my point `25 - q`.
- `legal_moves(state, d1, d2)` returns every distinct legal move, applying the "use both dice" and "play the larger die" rules. Each `Move` holds the steps, the resulting state (mover's view) and an outcome (`None`, or points won).
- `state.flip()` hands the position to the other player.

## Neural-network bot (`engine-rs/`)

A Rust engine (no dependencies) with the same move generation as the Python one (cross-checked in `tests/test_rust_movegen.py`), plus a TD(0) self-play trainer following the Palamedes recipe.

Build:

```bash
cd engine-rs && cargo build --release
```

Train (resumable; continues from `nets/td-latest.net`):

```bash
./run_training.sh
```

Compare two players (a net file, `heuristic` or `random`):

```bash
./target/release/mahbusa-rs bench --a nets/td-best.net --b heuristic --games 2000
```

Opening playouts:

```bash
./target/release/mahbusa-rs opening --net nets/td-best.net --games 4000 --top 3
```

Other commands: `replies`, `pinvalue`, `stats`, `choose`, `movegen`.

Play against the trained net from Python:

```bash
.venv/bin/mahbusa play --bot net
```

Results so far are in `analysis/`. After 2M training games the net beats the heuristic bot 81.5% of the time (+1.06 points per game), and its opening playouts match the Palamedes book on 19 of 21 rolls.

## Plan

1. ✅ Rules spec (`research/RESEARCH.md`)
2. ✅ Strategy guide v1, built from existing research
3. ✅ Game engine (Python) with tests, terminal play and baseline bots
4. ✅ Neural network: Rust engine and TD(0) self-play (2M games; `nets/td-best.net`)
5. ✅ Strategy guide v2 (`guide/STRATEGY_GUIDE.md`), with the bot's opening, reply, pin-value and risk results
6. ✅ Web board (`web/`): play the bot in any browser
7. ✅ Pin-lifting rules (guide §7, data in `analysis/pins/`)
8. ⏭ More worked example positions (guide §11)
