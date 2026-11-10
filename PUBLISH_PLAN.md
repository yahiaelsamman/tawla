# Publish plan: review findings and work list

Written 10 Nov 2026 after a line-by-line review of the engine, trainer, analysis commands, the guide and `REVIEW_BRIEF.md`. This file is a working document for whoever executes the plan. Delete it (or fold what's left into `docs/`) when the work is done.

---

## 0. Ground rules for the executor

- Work in `/home/user/tawla`. Do the work on a branch named `publish-prep` and push it; merge to `main` only when the owner says so. Don't open a pull request unless asked.
- Commits must be authored by the repo's local git identity (already configured; check with `git config user.name`). Never use a global identity. Don't add co-author or tool-attribution lines to commits, and don't mention any AI assistant or vendor anywhere in the repo (files, commit messages, comments).
- The owner's email address is private. Don't write it into any file or send it to any service.
- **Don't change `engine-rs/src/features.rs` or the forward pass in `nn.rs`**, and don't replace `engine-rs/nets/td-best.net` or `web/net.js`. Every number in the guide comes from that net and that encoding.
- Setup after a fresh container:

  ```bash
  cd /home/user/tawla && python3 -m venv .venv && .venv/bin/pip install -e ".[dev]"
  ```

  ```bash
  cd /home/user/tawla/engine-rs && cargo build --release
  ```

- After every code change: `.venv/bin/pytest -q` must show all tests passing (74 today) and `cargo build --release` must finish with no warnings.
- Speed: net-vs-net `bench` runs about 129 games/s on 4 threads, so 10,000 games ≈ 80 s and 100,000 games ≈ 13 min.
- Containers are reclaimed when idle. Launch anything longer than a few minutes detached (`setsid nohup ... > log 2>&1 < /dev/null &`), write results to files in the repo, and commit and push results as they arrive. Checkpoint files `engine-rs/nets/td-*.net` other than `td-best.net` are git-ignored and may vanish.
- Keep old result files. New results go in new files or folders, and the guide's numbers change only when a new result replaces them.

---

## 1. Verdict

**The engine, the rules, the network and the trainer are implemented correctly. Nothing in them needs fixing before publishing.** The approach (a TD(0) self-play net used as an oracle through rollouts) is the established method for this game family and is right for a human strategy guide.

What needs work is around it: repo hygiene (the repo is already public), a few small code and script defects, guide wording that claims more than the evidence shows, and some cheap experiments that close the biggest evidence gaps.

### What was checked and found correct (no need to redo)

| Area | Checked | Result |
|---|---|---|
| `engine-rs/src/state.rs`, `src/mahbusa/state.py` | pin, release, blocked points, mother win, bear-off (all home, none pinned, exact or higher die from the highest point), max-dice rule, larger-die rule, early game end, dedupe | correct, and identical in both engines |
| `features.rs` | shot counting (direct, combined with an open intermediate point, doubles up to 4 steps), blot definition, race flag, both sides encoded | correct |
| `nn.rs`, `train.rs` | forward pass, backprop, TD(0) target (next position inverted to the mover's view, terminal targets), pass handling, resume | correct |
| `play.rs`, `main.rs` | game loop, tallies, rollout sign conventions, rule players, evalset sign flip | correct, except the items in §2.2 |
| Guide §4 shot table and opening safety map | recomputed by hand | all entries correct |
| Guide §9 match table | recomputed with a simple recursion using a 40.5% double-win rate | all six entries match |
| Guide §5 lone-mother costs | compared with the formula (shots/36) × (2 + your equity) | consistent (see G5) |
| Guide §5 "70% mother pins with random play" | `mahbusa-rs speed` | 69.7% |
| Guide §6, §12 tables | compared with `analysis/net2m/` and `analysis/experiments/rules.txt` | match |
| External sanity of the net | first-player edge +0.110 (Palamedes +0.104), 19/21 openings, double-win rate 42.8% (Palamedes 40.5% under Greek rules) | consistent |

### The earlier decision to stop training at 2M games

Agreed, with one correction to the reasoning. The 2M→3M step used a learning rate of 0.01, three times lower than the step before it, so "no gain" partly reflects the schedule and isn't proof that the architecture is exhausted. It doesn't matter for the guide: the findings the guide relies on are effects of 0.15 points per game or more, and a net 0.05 stronger wouldn't change them. Say "plateaued under this training setup" (G10) and don't train further for now.

---

## 2. Must do before promoting the repo

### 2.1 Repo hygiene

| # | Task | Details |
|---|---|---|
| H1 | **Add a licence** (owner decides; see §6) | With none, nobody may legally reuse the code or the guide. Suggested default: MIT for code (`LICENSE`), CC BY 4.0 for `guide/`, `research/RESEARCH.md` and `analysis/` (`LICENSE-docs`, plus one line in the README). |
| H2 | **Remove `PROGRESS.md`** | It's an internal work log ("The user is asleep…", container restarts, scheduling notes). Nothing in it is needed by a reader; the history is in git. Remove README references to it. |
| H3 | **Turn `REVIEW_BRIEF.md` into `docs/METHODS.md`** | Title: "Methods, evidence and limitations". Keep §2–§8 and §10. Drop the "from the project owner" framing in §1, the sentence about the owner being new to neural networks, and the two "Operational notes" bullets (cloud containers, commit authorship). Rename §9 to "Possible future work". Update it with the results of §4 below. Update the README link. |
| H4 | **Delete stray run markers and logs** | `analysis/DONE`, `analysis/console.log`, `analysis/net2m/DONE`, `analysis/net2m/console.log`, `analysis/experiments/positions.done`. Stop the scripts writing them into `analysis/` (`run_analysis.sh` last line, `run_position_tests.sh` last line). |
| H5 | **Move the 1M-net results into `analysis/net1m/`** | The top-level files `opening_book.tsv`, `opening_rollouts.tsv`, `pin_value.tsv`, `pin_value_clean.tsv`, `quiz_candidates.tsv`, `quiz_verified.txt`, `releases.txt`, `replies_1ply.tsv`, `selfplay_stats.txt` are from the older 1M net and are easy to mistake for the guide's data. Use `git mv`. Update `analysis/README.md`, the docstring in `scripts/show_quiz.py`, and the default `OUT` in `engine-rs/run_analysis.sh` (make it `../analysis/net2m`). Grep for other references. |
| H6 | **Rewrite `README.md` for a visitor** | Order: one-paragraph description; "Read the guide" and "Play in the browser" links; five headline findings (take them from the guide's cheat sheet); how the findings were produced, in three sentences, with a link to `docs/METHODS.md`; quick start (tests, build, bench); repo map; credits (the Palamedes papers by Papahristou & Refanidis, Mamoun's primer, the other sources in `research/RESEARCH.md`); licence. Remove the "Plan" checklist. State plainly that the bot is strong but not perfect and hasn't been checked against Palamedes beyond the opening. List all subcommands. |
| H7 | **Continuous integration** | Add `.github/workflows/ci.yml` (ubuntu-latest; official `actions/checkout`, `actions/setup-python`, `actions/setup-node` only): `cargo build --release` in `engine-rs`, `pip install -e ".[dev]" && pytest -q`, then the JS check from H8. The Playwright test stays local. If the push is refused because the token lacks workflow permission, leave the file out of the push and tell the owner. |
| H8 | **Make the JS-vs-Rust check a script** | The check exists (`web/tests/check_engine.js`) but its inputs were generated by hand. Add `web/tests/run_engine_check.sh` that: builds about 5,000 `me;op;mpin;opin;d1;d2` lines with `collect_positions` from `tests/test_movegen_bruteforce.py` (same line format as `tests/test_rust_movegen.py`); pipes them through `mahbusa-rs movegen` and `mahbusa-rs choose --net nets/td-best.net`; runs `node web/tests/check_engine.js cases moves choose engine-rs/nets/td-best.net`. Also add a check that the weights embedded in `web/net.js` decode to exactly the bytes of `engine-rs/nets/td-best.net`. Write temporary files with `mktemp -d`. |
| H9 | **Play link** (owner decides; see §6) | If the owner enables GitHub Pages from `main` (root folder), the board is at `https://<user>.github.io/tawla/web/`. Put that link at the top of the README once it works. `web/index.html` loads fonts from Google Fonts and nothing else external; that's fine. |

### 2.2 Small code and script defects

| # | File | Defect | Fix |
|---|---|---|---|
| C1 | `engine-rs/run_analysis.sh`, `run_position_tests.sh` | They call `./target-analysis/release/mahbusa-rs`, which doesn't exist in a fresh clone, so the documented reproduction steps fail | `B=${BIN:-./target/release/mahbusa-rs}` in all three run scripts |
| C2 | `engine-rs/run_rule_tests.sh` | Writes to fixed names in `/tmp` | Use `mktemp -d` and clean up |
| C3 | `engine-rs/src/main.rs`, `cmd_pinvalue` | The doc comment says the pinner has just moved and the opponent is on roll. In fact the pinner is on roll and the printed equity is the **pinned** side's. The column names hide this, and the first analysis pass tripped on it | Fix the comment. Rename the header columns to `pinned_side_equity`, `pinned_side_gain`, `pinned_double_win%`, `pinned_double_loss%`. Don't change the numbers or the row layout: `scripts/summarize_analysis.py` reads the rows by position and already flips the sign |
| C4 | `engine-rs/src/main.rs`, `main()` | The usage line lists 5 of the 15 subcommands | List all, one line each |
| C5 | `engine-rs/src/play.rs`, `Tally::summary` | Prints a confidence interval for the win rate but not for points per game, which is the number the guide quotes | Add `±` for ppg. Per-game results are ±1 or ±2, so with `d = doubles[0] + doubles[1]` and `n = games`: mean square = `(n + 3d)/n`, variance = mean square − ppg², print `1.96 * sqrt(variance / n)` |
| C6 | `src/mahbusa/state.py`, Greek option (`Rules(mother_exception=True)`, `--plakoto`) | Incomplete. When the pinner still has checkers on its own start the game correctly continues, but the deferred double win is never awarded once the pinner's start is empty. The game just runs on to the bear-off. The default Egyptian rules are unaffected, and no result in the guide uses this option | In `_step`, after the move is applied and only when `rules.mother_exception` is set: if `outcome is None`, bit 1 of `mpin` is set (I hold the pin on their start) and `me[START] == 0`, set `outcome = rules.mother_points`. Add two tests next to `test_mother_exception_plakoto` in `tests/test_rules.py`: the win is awarded on the move that empties the pinner's start; it isn't awarded while the start still holds a checker. Mention in the `Rules` docstring that the Rust and JS engines implement the Egyptian rule only |
| C7 | `engine-rs/src/train.rs` | The Hogwild trainer shares `&mut Net` across threads with no synchronisation. It works in practice and is the standard trick, but it's formally undefined behaviour in Rust and training isn't bit-reproducible | Add a short comment at `Shared` saying so, and that `--threads 1` gives deterministic training. Say in `docs/METHODS.md` that the committed net can't be regenerated bit-for-bit, while every analysis run from it is seeded and repeatable |

---

## 3. Guide corrections (wording that outruns the evidence)

Apply these to `guide/STRATEGY_GUIDE.md`, bump it to v2.3 with the date, and keep the existing structure.

| # | Where | Problem | Change to |
|---|---|---|---|
| G1 | §12 intro: "how much each rule gives away against perfect play" | The opponent is our bot, which is strong but not perfect | "against the unrestricted bot" |
| G2 | §9 table row and §11 intro: the rule-based bot "plays like a careful club player" | It was never calibrated against human players | "our simple rule-based bot (a stand-in for a careful, inexperienced player; not measured against real players)" |
| G3 | §2 item 9: a counter-pin "roughly doubles your winning chances" | The 57% vs 25% figure is a correlation; §8 already says so | "Players who had one won twice as often (§8); the example in §11 shows a swing of 2 points." |
| G4 | §2 item 8 and the Mistake 3 heading ("leaving your last two checkers on the start") | They contradict §5, where the measurement says two checkers on the start are as safe as none and there's no rush. In the Mistake 3 position six of X's checkers are pinned, so X would soon be forced to break the pair | Item 8: "**Never leave exactly one checker on your start while anything can still reach it.** Move the last two together." Mistake 3 heading: "Missing the chance to clear your last two start checkers when you're running out of other moves". Add one sentence: "Six of your checkers are pinned here, so you'll soon be forced to move one of the pair alone. Otherwise two checkers on the start are safe (§5)." |
| G5 | §5 rule of thumb "0.07 points for every roll that can pin it" | It's stated as an empirical constant; it's really arithmetic, and it varies with the position | Add, tagged [Math]: "This is just arithmetic: a pinned mother turns your current prospects into −2, so each pinning roll costs (2 + your expected result) ÷ 36. That's 0.06 in an even game and 0.07 when you're ahead. Our playouts match." |
| G6 | §7: 3%, 97%, 41%, 46%, 6% and the "when never to lift" table | All come from the bot's one-move look-ahead scores, not from playouts | Until experiment E2 is done, add after the first paragraph: "These percentages use the bot's one-move look-ahead judgement of each move; the rule as a whole was then confirmed in full games (§12)." Replace with playout-checked numbers after E2 |
| G7 | §3 replies: "Each of these pins is worth about +0.9 points" and the 302/315 count | One-move look-ahead only | After E2c, replace with playout numbers. Until then add "(by the bot's one-move look-ahead)" |
| G8 | §12 table | No error bars; −0.02 and −0.03 aren't distinguishable from zero at 10,000 games (95% interval ≈ ±0.03) | After E1, show each cost with its ± and state the game count |
| G9 | §6 pin-value table intro | The positions are artificial: the pinning checker is placed on the point for free, which also gains it up to 22 pips | Add: "These positions are artificial (the pinning checker is simply placed there), so read the values as a guide to relative worth, not exact prices." |
| G10 | §12 "More training?" | See §1 | "…no stronger than the 2-million one, at a lower learning rate. The network has plateaued under this training setup." |
| G11 | Tag table, [Our bot] row | Doesn't say how far the bot can be trusted | Add: "It's strong, not perfect: it looks one move ahead, and outside the opening it hasn't been checked against another strong program. See `docs/METHODS.md`." |

---

## 4. Experiments that close evidence gaps

Ordered by value for cost. E1 and E2 should be done before promoting the guide. E3 and E4 are recommended. E5 and E6 are optional.

### E1. Rule tests with enough games (about 1.5 h of compute, no new code beyond C5)

At 10,000 games the standard error is 0.015, so the two "free" rules (never lift −0.02, never leave a lone start checker −0.03) and the trade-up variant can't be told apart from zero or from each other.

- Run 100,000 games each for `nolift`, `nolift_tradeup`, `pairs`, `homepin`, `alwayspin` with a new seed (the script uses 41; use 42). The other rules cost 0.29 or more and are already clear at 10,000.
- `run_rule_tests.sh` hard-codes the seed; make it `--seed ${SEED:-41}` and write to `${OUT:-../analysis/experiments/rules.txt}`. Write the new results to `analysis/experiments/rules_100k.txt`.
- Then update guide §12 (G8) and the README of `analysis/experiments/`.
- Expected outcome and how to word it: if `nolift` is within ±0.01 of zero, say "free". If `nolift_tradeup` isn't better than `nolift`, the guide's "one exception worth thinking about" has no support from full games; reduce §7 Rule 2 to a short note (and see E2a).

### E2. Check the one-move look-ahead against playouts (about 2.5 h of compute, one new command)

Three sets of guide claims rest on the net's static one-move scores: the lifting percentages (§7), the size of the "mistakes" database, and the replies table (§3). This experiment measures how often those scores agree with playouts.

**New command `verify` in `engine-rs/src/main.rs`.** Input on stdin, one line per case: `label<TAB>me;op;mpin;opin<TAB>d1d2<TAB>mode`, where `mode` is `lift:<pin_point>` or `quiz`.

- For `lift:<p>`: generate the legal moves, score each non-winning move with `NetPlayer::move_value`, take the best move that keeps the pin on `p` and the best that lifts it (the same selection as `cmd_liftdata`). Skip the case if either is missing or a winning move exists.
- For `quiz`: take the net's choice and `HeuristicPlayer`'s choice (the same as `cmd_quiz`). Skip if they're the same move.
- Roll out both moves with `rollout(&net, mv.after.flip(), games, seed, threads)`; it returns the equity of the player who made the move. Default `--games 1000`.
- Output: `label, moveA, moveB, netA, netB, rollA, seA, rollB, seB`.

**Sampling script `scripts/sample_for_verify.py`** (fixed random seed, writes the stdin file):

- **E2a, lifting.** From `analysis/pins/lift_decisions.tsv.gz` (columns are in the header; the position is in the last two columns `me` and `op_mpin_opin`): 100 cases where `lift_eq − keep_eq ≥ 0.05`; 100 trade-up candidates (`pin_point` is 5 or 6 and the lift move's last landing point is 2–4; confirm in `verify` output that the lift move really makes a new pin there); 100 drawn at random from the rest. 300 cases × 2 moves × 1,000 games ≈ 80 min.
- **E2b, mistakes.** 100 random rows of `analysis/net2m/quiz_candidates.tsv`. ≈ 26 min.

**Analysis script `scripts/summarize_verify.py`** → `analysis/experiments/verify_summary.txt`:

- Share of cases where the playout prefers the same move as the one-move score, overall and by size of the one-move margin (under 0.1, 0.1–0.25, over 0.25). Count a case as "agree", "disagree" or "too close" (playout difference under 2 combined standard errors).
- For the trade-up group: in what share does the playout say lifting was right? That number replaces the guide's 41%.
- For the quiz group: the share confirmed. If it's 90% or more, the guide can say the mistake themes hold across a random sample, not just six hand-picked positions.

**E2c, replies.** No new code: `mahbusa-rs replies --net nets/td-best.net --first 31,62,65,55 --games 2000 --top 3 > analysis/experiments/replies_rollout_sample.tsv` (about 1 h). Compare with the same rows of `analysis/net2m/replies_1ply.tsv`: how many best replies change, and what the 5-5 and 6-6 pin replies are worth by playout. Then fix G7.

Record the agreement rates in `docs/METHODS.md` under the one-move look-ahead weakness. If agreement at large margins is under about 85%, stop and report to the owner before editing the guide: it would mean the one-move claims need rework, not rewording.

### E3. Do the rules help a weak player? (about 20 min of compute, small code change)

Today each rule of thumb is tested only as a restriction on the strong bot playing itself. Such a test can show a rule is cheap; it can never show a rule *helps*, and the guide's readers aren't the strong bot.

- In `play.rs`, let `RulePlayer` wrap either base player. Add the spec `hrule:<name>`: the heuristic bot restricted to rule-obeying moves (pick the allowed move with the best `HeuristicPlayer::evaluate`, winning moves first). Keep `rule:<name>:<net>` unchanged.
- Run `hrule:<name>` against plain `heuristic`, 20,000 games each, for `nolift`, `nolift_tradeup`, `pairs`, `homepin`, `alwayspin`, `safe`, `safe12`, `nobury2`, `nocrash2`. Write to `analysis/experiments/rules_weak_player.txt`.
- Add a third column to the guide §12 table: "Effect for a simple player". A rule that costs the strong bot nothing and gains the weak bot points is the guide's best kind of advice, so say which those are.

### E4. A second, independent network (about 3.5 h of compute, no new code)

The brief's main weakness is that one self-trained net is the judge for almost everything. Palamedes is closed-source Windows software, so the practical second opinion is a net trained separately.

- Train with a different seed and a separate file prefix so `td-latest.net` isn't touched. Loop in 100k chunks like `run_training.sh` does (copy it to `run_training_alt.sh` with `--out nets/alt`, seed offset +5000, learning rate 0.1 to 800k games then 0.03 to 1M). Launch detached; it resumes from `nets/alt-latest.net`.
- When done, commit the result as `engine-rs/nets/td-alt.net` (add `!nets/td-alt.net` to `engine-rs/.gitignore`).
- Then, with `td-alt.net`: (1) `bench` it against `td-best.net`, 20,000 games; (2) rerun the rule tests at 10,000 games with `NET=nets/td-alt.net` (make the script take `NET` from the environment); (3) rerun `run_position_tests.sh` (mother, standoff, bear-off stacking) into `analysis/experiments/alt/`; (4) run `compare` on the six Mistake positions and the trade-up example (positions are in `analysis/net2m/quiz_verified.txt`, `examples2_verified.txt` and `analysis/pins/tradeup_verified.txt`).
- Write `analysis/experiments/replication.md`: one table, finding by finding, "2M net" vs "independent net", and whether the sign and rough size hold. Summarise in `docs/METHODS.md` and in one sentence in the guide's tag table.
- If the checkpoints `td-1000k.net` and `td-3000k.net` still exist locally, they can be used for a quick first look, but they share the 2M net's training history and don't count as independent.

### E5. Two-move look-ahead (optional; about half a day including code)

- Add a 2-ply player: rank the moves by the 1-ply score, keep the top 5, and for each one average over the 21 distinct opponent rolls (weights 1 or 2 out of 36) the result of the opponent's best 1-ply reply; pick the best average. Expect roughly 100 times the cost of 1-ply.
- Measure 2-ply vs 1-ply over 2,000 games (Palamedes reported about +0.07). Then re-check with 2-ply move choice: the 21 openings, the Mistake positions, and the E2 samples where the playout and the one-move score disagreed.

### E6. Score-aware play in a set to 3 (optional; about 2 h including code)

The net already outputs win, double-win and double-loss chances, so choosing moves by set-winning chance is a small change: value = Σ over the four outcomes of probability × the §9 table entry for the resulting score. Play 20,000 sets of score-aware bot vs plain bot and count how often the score changes the chosen move. This would turn guide §9's advice from reasoning into measurement. Palamedes found only about a 1% gain, so expect a small effect.

### Not planned

- **More training of the same net, bigger nets, endgame databases.** They'd make a stronger bot, which is the secondary goal; they wouldn't change the guide.
- **Checking against Palamedes (Polygammon).** It needs a person at a Windows machine entering positions by hand. Worth doing if the owner wants: 20 positions from guide §11 and the E2 disagreements would be enough.

---

## 5. Order of work and definition of done

1. H1–H8, C1–C7 (one commit per group). Tests pass, build is warning-free, the scripts run from a fresh clone with `./target/release/mahbusa-rs`.
2. G1–G5, G9–G11 (wording only).
3. E1, then G8. E2, then G6 and G7.
4. E3, E4 (start the E4 training early, detached; it runs alongside everything else, though benchmarks will be slower while it does).
5. Update `docs/METHODS.md` with what E1–E4 showed; final README pass; delete this file.
6. E5 and E6 only if the owner asks.

Done means: a visitor can read the README, open the guide, play the board, run the tests and reproduce any table in the guide with a documented command; every number in the guide either comes from playouts with a stated sample size or is labelled as a one-move judgement; and no file contains internal work-log material.

---

## 6. Decisions for the owner

| # | Decision | Suggested default |
|---|---|---|
| D1 | Licence | MIT for code, CC BY 4.0 for the guide and data |
| D2 | The commit history shows the owner's personal email address as author, and the repo is already public | Either accept it, or switch the repo's git identity to the GitHub no-reply address and rewrite the five commits (needs a force-push; do it before anyone forks) |
| D3 | Enable GitHub Pages so the board can be played from a link | Yes, from `main`, root folder (a repo setting only the owner can change) |
| D4 | `research/RESEARCH.md` says the rules were "confirmed by Yahia" | Keep, or change to "confirmed by the project owner" |
| D5 | An Arabic version of the guide | Worth doing after the corrections in §3 land, since the audience is Egyptian players; not required for publishing |
| D6 | Run the optional experiments E5 and E6 | Not needed for publishing |
