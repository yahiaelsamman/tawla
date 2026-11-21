# Look-ahead (2-ply)

The guide's bot picks moves at 1-ply: it plays each legal move and asks the network how good the resulting position is. Here we add one more roll of look-ahead. That gives a stronger bot and a better check on the guide's claims. We also tried to teach the look-ahead to the network itself, which didn't work.

**2-ply.** For each candidate move, go through all 21 possible opponent rolls. For each roll, let the opponent pick its best reply by 1-ply value, and average the results (doubles count once, other rolls twice, out of 36). Looking at every legal move this way would be about 100 times slower than 1-ply. So the moves are first ranked at 1-ply, and only the best 5 within 0.2 points of the top move are searched (a "move filter"). Moves far below the best at 1-ply practically never come out best at 2-ply. Code: `engine-rs/src/search.rs`; players `ply2:<net>` and `rule2:<rule>:<net>` in `bench`. The browser engine has the same search (`web/engine.js`, `rankMoves2`), checked against Rust by `web/tests/check_ply2.js`.

## Results

| Question | Result | File |
|---|---|---|
| Is 2-ply stronger than 1-ply (same net)? | Yes: +0.075 ± 0.066 over 2,000 games and +0.051 ± 0.047 over 4,000; pooled about **+0.06 ± 0.04 points per game**. Palamedes reported +0.07 for the same step. | `bench_ply2_vs_ply1*.txt` |
| Is 2-ply closer to the truth than 1-ply? | On the 400 decisions with 1,000-game playouts (`analysis/experiments/verify/`), the error in judging which of two moves is better drops from 0.139 to 0.099 points, and agreement on decidable cases rises from 269 to 272 of 291. | `verify_vs_2ply.txt` |
| Opening book | 2-ply agrees with Palamedes on 20 of 21 first rolls (1-ply: 19). It now picks the book move for 2-2; the remaining difference, 5-5, was already settled for the book move by the guide's rollouts. | `opening_2ply_best.tsv` |
| Guide example positions (§7, §11) | Same best move at 2-ply in all ten. | `examples_2ply.txt` |
| Rules of thumb, with look-ahead on both sides (1,500 games each, ±0.08) | Never leave a 6+-shot blot: −0.33 (1-ply: −0.34). While pinned, don't bring checkers home: −0.30 (1-ply: −0.32). Never add to your 1–2 points: −0.25 (1-ply: −0.29). The guide's conclusions stand. The cheap rules (costs of 0.02–0.05) would need far more games than 2-ply allows. | `rules_2ply.txt` |

## 3-ply, pruning and speed

**Faster evaluator.** Most of the 253 inputs are zero for any position, so the first layer is now stored transposed and only the columns of non-zero inputs are added up (`Net::prepare` in `nn.rs`; the browser engine does the same). Outputs are identical to the last printed digit, and everything runs about 8 times faster: 1-ply games went from about 130 to about 1,000 per second, and 2-ply-vs-2-ply games from about 1.5 to 12 per second (4 cores). Training is unchanged; it uses the plain layout.

**How much the move filter costs** (`filtercheck.txt`, 6,365 decisions from 400 self-play games; every legal move evaluated at 2-ply to find the unfiltered choice):

| Filter | Choice differs from searching every move | Average 2-ply loss per decision | Moves searched per decision |
|---|---|---|---|
| none (1-ply only) | 22.7% | 0.0034 | 0 |
| keep 2 within 0.06 | 10.9% | 0.0013 | 1.4 |
| keep 3 within 0.12 | 6.0% | 0.0005 | 2.4 |
| **keep 5 within 0.2 (used)** | **2.9%** | **0.0003** | **3.9** |
| keep 8 within 0.3 | 1.5% | 0.0001 | 5.8 |

The filter in use loses about 0.0003 points per decision, roughly 0.015 per game, for a 5-6x saving. Tighter filters save little time and lose more.

**3-ply.** One more roll: the opponent's 21 rolls, each answered with the opponent's best reply chosen at 2-ply among its 1-ply top 3 within 0.12 (a reply that is the only survivor keeps its 1-ply value, which saves time). A 3-ply decision runs the 1-ply filter, evaluates the survivors at 2-ply, and searches the best 3 within 0.1 at 3-ply. Player `ply3:<net>`; columns `--ply3 1` in `compare` and `verify`.

| Question | Result | File |
|---|---|---|
| Is 3-ply stronger than 2-ply? | Yes: **+0.071 ± 0.047** per game over 4,000 games, about as big a step as 1-ply to 2-ply. | `bench_ply3_vs_ply2.txt` |
| Closer to the playouts? | On the 400 playout-checked decisions: right in 269 / 272 / 276 of the 291 clear cases at 1 / 2 / 3-ply; average error comparing two moves 0.139 / 0.099 / 0.087 points. (The playouts themselves were played at 1-ply.) | `verify_*_3ply.tsv` |
| Openings | 20 of 21 rolls match Palamedes, as at 2-ply. | `opening_3ply_best.tsv` |
| Guide example positions | Same best move at 1, 2 and 3-ply in all ten. | `examples_3ply.txt` |
| Speed | About 25 ms per move evaluated at 3-ply in Rust; in the browser a 3-ply decision takes a median 0.4 s and up to about 5 s. | |

**Self-play statistics with the 2-ply bot** (`selfplay_stats_2ply.txt`, 10,000 games) match the 1-ply ones the guide uses: pins on the 2-, 3- and 4-point win 90%, 79% and 68% (1-ply: 93%, 81%, 69%), double wins 43.5% (42.8%), and the start is emptied by turn 13 (13). The 2-ply bot leaves slightly riskier blots in the middle game, which supports the guide's "calculated risk" point.

## Teaching the look-ahead to the network (distillation): no gain

We generated about 600,000 positions from self-play (half of them positions after alternative moves, so the net also sees the positions it has to compare), labelled each with its 2-ply value, and trained the net toward those labels (`mahbusa-rs gen2`, `mahbusa-rs fit`).

- The 1-ply net was already close to its own 2-ply value: an average gap of 0.033 points, which training reduced to 0.025 and no further (`fit_d1.txt`). A net with twice the hidden units stopped at the same level (`fit_w1.txt`), so the limit isn't network size.
- Playing strength didn't change. Round 1 looked like +0.022 ± 0.021 over 20,000 games, but 40,000 fresh games gave −0.002 ± 0.015 (`bench_d1_vs_best.txt`). Round 2, trained on both rounds' labels, gave −0.008 ± 0.021 (`bench_d2_vs_best.txt`).

So the look-ahead gain comes from searching at decision time, not from a better static evaluation. The distilled nets were not kept; `td-best.net` is unchanged.

## Reproducing

```bash
cd engine-rs && cargo build --release
./target/release/mahbusa-rs bench --a ply2:nets/td-best.net --b nets/td-best.net --games 4000 --seed 22
./target/release/mahbusa-rs opening --net nets/td-best.net --games 0 --top 5 --ply2 1
./run_rule_tests_2ply.sh 1500
./target/release/mahbusa-rs gen2 --net nets/td-best.net --games 14000 --sample 0.25 --out d1.tsv
./target/release/mahbusa-rs fit --net nets/td-best.net --data d1.tsv --epochs 3 --alpha 0.01 --out td-d1.net
```
