# Progress log

The user is asleep and asked for autonomous iteration until they say stop. Resume messages are scheduled at 01:40 and 07:00 Europe/Berlin.

## Done
- Research (`research/RESEARCH.md`), guide v1 (`guide/STRATEGY_GUIDE.md`), Python engine + 72 tests.

## Current phase: neural-network bot
Plan:
1. [x] Rust core in `engine-rs/` (no external crates): state, move generation (port of the Python engine), features, NN, TD(0) trainer, benchmark, rollouts.
2. [x] Cross-check Rust move generation against the Python engine (`tests/test_rust_movegen.py`).
3. [~] Train the TD(0) net in the background (Palamedes recipe: 1 hidden layer, 3 outputs W/WD/LD, λ = 0). Checkpoints go in `engine-rs/nets/`.
4. [ ] Benchmark against random, heuristic and earlier checkpoints.
5. [ ] Analysis with the trained net: opening rollouts under Egyptian rules (compare with the Palamedes book), second-player replies, pin value by point, release timing.
6. [ ] Guide v2.

## Notes
- 2026-11-06 00:50 CET: Rust engine built (about 1,700 random games/s per thread). It matches Python on about 10k positions (`tests/test_rust_movegen.py`).
- A smoke test showed the net equal to the heuristic bot after only 4k games. Training speed is about 95 games/s on 4 threads.
- Training was launched DETACHED (setsid nohup) with `engine-rs/run_training.sh`: stage 1 is 1M games at α 0.1, stage 2 is +1M at α 0.03. That's about 6 h in total.
  Logs: `engine-rs/nets/td.log` (one line per checkpoint, with a benchmark against the heuristic) and `engine-rs/nets/training_console.log`.
  Check it's alive with `pgrep -af mahbusa-rs`. Checkpoints are `nets/td-<N>k.net`; `nets/td-latest.net` is the newest.
- Do NOT change features.rs or nn.rs while training runs, because stage 2 reloads the net with the same encoding. Build analysis tools with
  `CARGO_TARGET_DIR=target-analysis cargo build --release` so the training binary isn't replaced.
- 01:05 CET: the 50k-game checkpoint beats the heuristic 71% (+0.745 ppg). Added `mahbusa play --bot net` (Python front-end talks to `mahbusa-rs choose`).
  1-ply openings at 50k agree with the Palamedes book on 14 of 15 non-doubles (5-1 differs). Doubles 22, 33, 44 and 55 differ, to recheck with later nets plus rollouts.
- Next, once stage 1 finishes (around 04:00 CET): run `opening` rollouts (top 3, 5k games), `pinvalue` and `replies` with nice priority on td-stage1.net, then write guide v2.
- 01:32 CET: the CONTAINER RESTARTED and killed training at 200k games (checkpoints survived).
  `run_training.sh` is now RESUMABLE: 100k-game chunks, resuming from nets/td-latest.net, with alpha chosen from the game count.
  After any restart, relaunch with: `cd engine-rs && setsid nohup ./run_training.sh >> nets/training_console.log 2>&1 < /dev/null &`
  Benchmark against the heuristic so far: 50k 71%, 100k 70%, 150k 74%, 200k 75% (+0.87 ppg).
- 02:02 CET: a second container restart (at 300k games). Restarts seem to happen while the session is idle. Relaunched with checkpoints every 25k, and staying active with foreground waits.
- 04:00 CET: training reached 1M games (td-1000k = td-stage1 = td-best). It beats td-500k by +0.11 ppg, td-200k by +0.19 and td-50k by +0.32 (still improving).
- 04:05–04:30: analysis batch (`engine-rs/run_analysis.sh`); results are in `analysis/`:
  selfplay_stats.txt, pin_value.tsv (raw, signs flipped: the pinner was on roll), pin_value_clean.tsv (corrected), replies_1ply.tsv, opening_rollouts.tsv, opening_book.tsv.
  The opening rollouts agree with Palamedes on 17/21 rolls; the other 4 (51, 53, 22, 55) are statistical ties. Total first-player equity is +0.10 (Palamedes 0.104).
- Guide v2 written (openings check, second-player replies, pin value table, pin stats, risk profile, unstacking speed).
- 04:30: stage 2 training resumed (alpha 0.03 toward 2M games).
- 04:50 CET: added `quiz` (positions where the heuristic errs, by net equity) and `compare` (rollouts of the top moves) to mahbusa-rs. Results: analysis/quiz_candidates.tsv and quiz_verified.txt.
  Guide §11 "three costliest mistakes" (take the deep pin, never lift a home pin, clear the last two start checkers together), each verified by 1,500-game rollouts.
- 05:00 CET: `releases` command added. Results in analysis/releases.txt: home pins are kept 97.6% of the time before the bear-off and 84–95% during it; deep-pinned players win 59% with a counter-pin and 23% without. Guide §7 and §8 updated.
- 06:40 CET: training finished at 2M games. td-final beats td-1000k by +0.061 ppg (10k games, 52.2%) and the heuristic by +1.06 ppg (81.5%). td-best.net is now td-final.
- 06:45: re-running the whole analysis batch with the 2M net into analysis/net2m/ (stats, pin value, replies, openings, releases).
- 07:35 CET: the 2M analysis is done (analysis/net2m/; summarised by scripts/summarize_analysis.py). Openings agree with Palamedes on 19/21 rolls (3-1 and 2-2 are near-ties); first-player equity is +0.110 (Palamedes +0.104).
  Guide v2 is updated to the 2M numbers, and all §11 example positions were re-verified with the 2M net.
- 07:40: stage 3 training started (alpha 0.01, TARGET=3M, ~3 h). td-best.net stays the 2M net; promote a newer one only if it wins a benchmark against td-best.
  NOTE: when it finishes, run_training.sh copies td-latest to td-final.net (overwriting the 2M copy; td-best.net and td-2000k.net still hold it).

## Morning summary (for the user)
- Guide v2: `guide/STRATEGY_GUIDE.md`. New: the bot's checks of the opening book, how to reply as second player, a pin-value table, pin statistics, risk profile, unstacking speed, release behaviour, counter-pin defence, and four costliest mistakes with diagrams.
- The bot (2M self-play games) beats the guide-v1 heuristic 81.5% of the time and agrees with Palamedes's opening book. Play it with `.venv/bin/mahbusa play`.
- Nothing is committed to git yet; waiting for the user's GitHub repo.

## Session 2 (6 Nov, afternoon): examples, pin-lifting rules, web board
- Stage-3 training had died at about 2.05M games during a restart; left off. td-best is still the 2M net.
- Web board: `web/index.html` + `engine.js` (JS port, identical moves to Rust on 5,321 positions; net equity difference ≤ 5e-5) + `net.js` (base64 weights).
  Tests: `node web/tests/check_engine.js ...` and `web/tests/play_test.js` (Playwright; plays full games). Also published as a private web page.
- Pin-lifting: `mahbusa-rs liftdata` gives 134k decisions (analysis/pins/lift_decisions.tsv). Rules are in analysis/pins/lift_rules.txt and guide §7:
  never lift (97% agreement); the exception is trading a 5/6 pin for a deeper 2–4 pin (41% right); verified example in tradeup_verified.txt.
- More examples: `quiz` with the 2M net (700 games, 4,983 candidates) and `scripts/categorize_quiz.py` (themes). Added guide §11 Mistake 5 (counter-pin when pinned)
  and Mistake 6 (burying checkers on 1–2), both rollout-verified (analysis/net2m/examples2_verified.txt). The guide is now v2.1.

## Session 3 (6–7 Nov): repo + more training
- Project moved into the GitHub repo yahiaelsamman/tawla (this repo); git identity is set per repo.
- Training continued from 2.05M to 3M games (alpha 0.01). Results:
  - 2.5M vs 2M: +0.026 ppg (6k games, 50.9%), not significant.
  - **3M vs 2M: +0.006 ppg (20k games, 50.1% ±0.7)**, no improvement. Against the heuristic: +1.077 ppg (82.1%).
  - Conclusion: this network (253 inputs, 100 hidden units, 1-ply) has plateaued. More of the same self-play doesn't help.
    td-best.net stays the 2M net (the guide's numbers come from it). The 3M checkpoint is kept locally (nets/td-3000k.net, not committed).
- Ideas for a stronger bot: 2-ply look-ahead (Palamedes measured about +0.07 ppg), a bigger hidden layer, more expert features (escape/prime), pin-endgame databases.
