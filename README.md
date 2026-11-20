# Mahbusa (محبوسة)

Mahbusa is the tawla game where you trap ("pin") your opponent's checkers instead of hitting them. Greeks call it Plakoto and Bulgarians call it Tapa. There's plenty written about regular backgammon strategy and almost nothing about this one, especially in Arabic, so this repo tries to work it out.

What's here:

- [The strategy guide](guide/STRATEGY_GUIDE.md). Each claim says where it comes from: the Palamedes research, expert advice, plain dice maths, or our own bot.
- [A board you can play in the browser](web/index.html) against the bot. Download the repo and open `web/index.html`; there's nothing to install. It also tells you after each move whether the bot would have played differently.
- [How the results were produced](docs/METHODS.md), including what could still be wrong with them.

## The short version

- On your first roll, move two checkers off the start, one with each die. Don't run one checker with both.
- Before you leave a checker alone, count how many of the 36 rolls can pin it.
- A pin deep in your home (your 2-, 3- or 4-point) is worth about a full point, often a double win. Take it even if it costs you a block.
- Don't lift a pin in your home board unless the dice force you. Pile checkers on it in the bear-off.
- If you get pinned, look for a pin of your own. In a pin standoff, the deeper pin usually wins.

## How it was done

The rules come from Arabic and Greek sources (see `research/RESEARCH.md`). The bot is a small neural network that learned by playing itself, following the method of the Palamedes Plakoto program. It trained for 3 million games, but it stopped improving after 2 million, so the 2-million-game version is the one used for the guide. It beats a hand-written rule-based bot 81% of the time and agrees with Palamedes' published opening moves on 19 of 21 rolls.

Most numbers in the guide come from setting up a position and letting the bot play it out thousands of times, or from making the bot follow a rule of thumb and seeing how much it loses. That play looks one move ahead. Adding a second level of look-ahead (2-ply: also considering every roll the opponent could get next) makes the bot about 0.06 points per game stronger, and spot checks with it agree with the guide (see `analysis/lookahead/`). The board in the browser plays at 2-ply. The bot is strong, not perfect. The weak spots are listed in `docs/METHODS.md`.

## Rules

Standard Egyptian rules, no house rules. Everyone starts with all 15 checkers on their 24-point and you move in opposite directions. Landing on a single enemy checker pins it until you leave. You can't bear off while one of your checkers is pinned. Pinning the opponent's last checker on their starting point (the "mother") wins double on the spot, and so does winning before the loser has borne anything off (mars). The full rules and sources are in `research/RESEARCH.md`. The Python engine can also play the Greek mother rule (`--plakoto`).

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
| `bench` | play two players against each other: a net file, `ply2:<net>` (2-ply), `heuristic`, `random`, `rule:<name>:<net>`, `rule2:<name>:<net>` or `hrule:<name>` |
| `opening` | rollouts of the best first moves for every roll |
| `replies` | best second-player reply for every pair of rolls |
| `pinvalue` | value of one early pin by point |
| `stats` | self-play statistics: pins, risk, how fast the start empties |
| `releases` | when home-board pins are released; deep-pinned players with or without a counter-pin |
| `liftdata` | keep-or-lift decisions for home-board pins |
| `quiz` | positions where the heuristic bot is much worse than the net |
| `compare` | roll out the best moves in one position |
| `evalset` | roll out a list of positions from stdin |
| `verify` | check 1-ply (and 2-ply) judgements against playouts |
| `gen2`, `fit` | label self-play positions at 2-ply and train a net toward them |
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

The research leans heavily on the Palamedes papers by Nikolaos Papahristou and Ioannis Refanidis (<https://github.com/nikpapa/palamedes>), which also supply the opening book and match table quoted in the guide, and on J. Mamoun's Plakoto primer. The other sources are listed in `research/RESEARCH.md`.

## Licence

Code: MIT (`LICENSE`). Written material in `guide/`, `docs/`, `research/RESEARCH.md` and `analysis/`: CC BY 4.0 (`LICENSE-docs`).
