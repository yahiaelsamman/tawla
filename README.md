# Mahbusa (محبوسة) strategy project

Mahbusa is the Egyptian pinning variant of tawla (backgammon). Internationally it is known as Plakoto (Greece) or Tapa (Bulgaria). This project works out what wins, using a self-trained neural network as a sparring partner and measuring each idea in thousands of games, and writes the results down for human players.

- **[Read the strategy guide](guide/STRATEGY_GUIDE.md)** (English). Every claim is tagged with how solid the evidence is.
- **[Play against the bot in your browser](web/index.html)**: open `web/index.html`, no install. A wooden tawla board, the trained bot, and a coach that rates your moves.
- **[Methods, evidence and limitations](docs/METHODS.md)**: how the findings were produced, and what could be wrong with them.

## Five things the guide says

1. **Split two checkers off the start on every non-double opening roll.** Never run one checker with both dice.
2. **Count the pin shots before leaving any blot.** How many of the 36 rolls pin it is the most important tactical number.
3. **A deep early pin is worth about a point, and on your 2- or 3-point close to a mars.** Take it, even at the cost of a block.
4. **Never lift a home-board pin on purpose.** Hold it into the bear-off and stack checkers on it.
5. **Pinned yourself? Hunt for a counter-pin.** And in a pin standoff, depth beats numbers.

## How the findings were produced

The rules were fixed from the sources in `research/RESEARCH.md`. A rule-based bot gave a baseline, and a neural network trained on 2 million self-play games (the published Palamedes recipe) gave a much stronger one. The guide's numbers come from rollouts of controlled positions, self-play statistics, and "rule of thumb" players: the network forced to obey a rule, compared with itself. The bot is strong but not perfect. It looks one move ahead, and apart from the opening book it has not been checked against another strong program. See [docs/METHODS.md](docs/METHODS.md) for details and the list of known weaknesses.

## Rules implemented (Egyptian Mahbusa)

All 15 checkers start on your 24-point (the opponent's 1-point). Players move in opposite directions. Landing on a lone enemy checker **pins** it until all your checkers leave that point. A pin or 2+ checkers make a block. Pinning the opponent's last checker on their start (the **mother**) wins at once, double. Bear off only when all 15 are home and none is pinned. A win is 1 point, or 2 (**mars**) if the loser has borne off nothing. Full rules are in `research/RESEARCH.md` §2. The Greek mother exception exists as an option in the Python engine (`--plakoto` or `Rules(mother_exception=True)`); the Rust engine and the browser board play the Egyptian rule only.

## Quick start

Python engine, tests and terminal play:

```bash
python3 -m venv .venv
.venv/bin/pip install -e ".[dev]"
.venv/bin/pytest
.venv/bin/mahbusa play
```

Rust engine and the trained net:

```bash
cd engine-rs && cargo build --release
./target/release/mahbusa-rs bench --a nets/td-best.net --b heuristic --games 2000
```

Play the net from the terminal:

```bash
.venv/bin/mahbusa play --bot net
```

Bot-vs-bot simulation in Python:

```bash
.venv/bin/mahbusa sim --a heuristic --b random -n 1000
```

### Rust commands (`engine-rs/`)

`mahbusa-rs <command> --key value ...`; run it without arguments for the list.

| Command | What it does |
|---|---|
| `train` | TD(0) self-play training, resumable (`run_training.sh`) |
| `bench` | play two players against each other: a net file, `heuristic`, `random`, or `rule:<name>:<net>` |
| `opening` | rollouts of the best first moves for every roll |
| `replies` | best second-player reply for every pair of rolls |
| `pinvalue` | value of one early pin by point |
| `stats` | self-play statistics: pins, risk, how fast the start empties |
| `releases` | when home-board pins are released; deep-pinned players with or without a counter-pin |
| `liftdata` | keep-or-lift decisions for home-board pins |
| `quiz` | positions where the heuristic bot is much worse than the net |
| `compare` | roll out the best moves in one position |
| `evalset` | roll out a list of positions from stdin |
| `motherstats` | how often Egyptian and Greek mother rules decide a game differently |
| `movegen`, `choose`, `speed` | helpers: cross-check move generation, serve the Python front-end, speed check |

Reproducing the tables in the guide: see the "Repository map and reproduction" section of [docs/METHODS.md](docs/METHODS.md) and the README in `analysis/experiments/`.

## Repository map

| Path | What it is |
|---|---|
| `guide/STRATEGY_GUIDE.md` | The strategy guide |
| `docs/METHODS.md` | Methods, evidence, limitations |
| `research/RESEARCH.md` | Rules, sources, existing strategy knowledge and computer analysis |
| `src/mahbusa/`, `tests/` | Python engine, rule-based bot, terminal play; rule tests and a brute-force cross-check of move generation |
| `engine-rs/` | Rust engine, trainer and analysis commands; the trained net is `nets/td-best.net` |
| `web/` | The browser board (engine port, net weights, coach) |
| `analysis/` | Result files: `net2m/` (the 2M-game net, used by the guide), `net1m/` (an older net), `experiments/`, `pins/` |
| `scripts/` | Shot table, quiz rendering, experiment builders, summaries |

## Credits

- The strategy and rules research builds on the Palamedes work of Nikolaos Papahristou and Ioannis Refanidis (papers at <https://github.com/nikpapa/palamedes>), including the opening book and the match-equity table the guide quotes, and on J. Mamoun's Plakoto primer.
- Other sources (Jawaker's rules, Egyptian Streets, tavli.net, boardgammon.com and more) are listed in `research/RESEARCH.md`.
- Mahbusa, Plakoto and Tapa are traditional games.

## Licence

Code: MIT (`LICENSE`). Written material in `guide/`, `docs/`, `research/RESEARCH.md` and `analysis/`: CC BY 4.0 (`LICENSE-docs`).
