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
