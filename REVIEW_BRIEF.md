# Review brief: Mahbusa strategy project

A self-contained summary of what has been built, every decision taken, the evidence behind each one, and the weak spots. It's written for a reviewer who wants to judge whether the approach is sound and what should be done differently. As of 10 Nov 2026, repo `yahiaelsamman/tawla`, branch `main`.

---

## 1. Goal and priorities (from the project owner)

- **Primary goal:** a **written strategy guide for humans** for Mahbusa (محبوسة), the Egyptian pinning variant of tawla/backgammon.
- **Secondary goal:** a strong bot. Nice to have, not the focus.
- The owner asked for simulation and/or machine learning to find what actually wins, not only to restate folk wisdom.
- House rules were explicitly out of scope; the game is played as standard Egyptian Mahbusa.
- The owner is new to neural networks and worried about losing momentum, so deliverables were kept incremental and visible (guide v1, then engine, then bot, then guide v2.x).

## 2. Rules (decisions and sources)

**Mahbusa = Plakoto (Greece) = Tapa-family pinning game.** Four independent sources say so: Mamoun's Plakoto primer ("in Egypt, where it is known as Mahbooseh"), Papahristou & Refanidis 2011 ("'Mahbusa' in some Arab countries"), Wikipedia (Plakoto/Tapa), and Mamoun's book subtitle. The Arabic and Egyptian write-ups (Jawaker's official rules, Egyptian Streets) match Plakoto point for point.

Implemented rules (all confirmed by the owner):

| Rule | Decision | Source / note |
|---|---|---|
| Setup | All 15 checkers on your 24-point (the opponent's 1-point); opposite directions; home = 1–6 | all sources |
| Start | One die each, higher starts, **then re-roll both dice** (opening doubles allowed); winner of a game starts the next | Plakoto rule; owner confirmed "like Plakoto" |
| Pinning | Landing on a lone enemy checker pins it until **all** your checkers leave; a pinner can't be pinned; a pin or 2+ checkers block a point | all sources |
| Mother | Pinning the opponent's last checker on its start = **immediate double win, no exception** | Jawaker (Egyptian/Arab); Greek Plakoto has an exception (configurable, `mother_exception`) |
| Forced moves | Use both dice if possible; if only one can be used, the larger one | standard; Plakoto sources |
| Bear-off | Only when all 15 are home **and none is pinned**; exact die, or a higher die from the highest point; bearing off is optional | Jawaker, Mamoun, bkgm |
| Scoring | 1 point; **mars** = 2 if the loser has borne off nothing; mother pin = 2; no triples, no cube | all sources |
| Match | Egyptian set = first to 3 points (Egyptian Streets) | the bot is NOT match-aware (see §8) |

Measured afterwards: the Egyptian vs Greek mother rule decides only **0.8% of games** differently (20,000 bot games), so this choice is low-stakes.

## 3. Research basis

- **Read in full:** Mamoun's free primer (bkgm.com PDF); all 6 Palamedes papers (github.com/nikpapa/palamedes/papers); Wikipedia (Plakoto, Tavli, Tapa); bkgm.com Plakoto/Tavli pages; Jawaker Arabic rules; Egyptian Streets; Egypt Independent; tavli.net; boardgammon.com; BrainKing; the Tzannes excerpt (jpneto). Local copies are git-ignored; notes are in `research/RESEARCH.md`.
- **Not accessible:** Mamoun's 220-page book *Plakoto Board Game Strategy* (2017), the Tzannes (1977) and Frantzis (1979) books. No serious Arabic strategy literature exists online.
- **Key external facts used:**
  - Palamedes's opening book (100k rollouts per candidate, Plakoto rules).
  - Plakoto gammon rate 40.5%; first-player equity +0.104.
  - The Palamedes network recipe: 1 hidden layer of 100 units, outputs W/WD/LD, TD(0) with λ = 0 (λ > 0.6 diverged), about 5M games to peak, plus expert features (pin probability per point, checkers in front of a pin, escape probability).
  - A Plakoto match equity table.
- **Palamedes (Polygammon / HappyGammon)** is a free, expert-level Plakoto program but not open source. It was never used directly to validate our bot. **That's a gap** (see §9).

## 4. Approach and why

1. **Guide v1 from the literature** first: a fast, motivating deliverable with every claim tagged [Palamedes] / [Expert] / [Math] / [To test].
2. **Python reference engine** (correctness first), then a **Rust engine** for speed (Python: about 90 random games/s; Rust: about 1,700/s per thread).
3. **Rule-based heuristic bot** (hand weights encoding guide v1) as a baseline and a "careful club player" proxy.
4. **TD(0) self-play network** following the published Palamedes recipe, chosen because it's proven on exactly this game family. Hand-tuned or evolved heuristics were judged a dead end: they can only rank strategies we think of.
5. **Use the network as an oracle** to answer the guide's open questions: rollouts of candidate moves, controlled positions, self-play statistics, and "rule players" (the net constrained by a rule of thumb).
6. Training was stopped at 2M games after the 3M network showed no gain. More compute on the same architecture was judged not worthwhile.

## 5. Software and how it was validated

| Component | Path | Validation |
|---|---|---|
| Python engine (rules, move generation, render, heuristic bot, terminal play) | `src/mahbusa/` | 74 pytest tests: hand-built rule tests, a brute-force move-generator cross-check on 1,000+ real positions, invariants over random games |
| Rust engine and tools | `engine-rs/src/` (`state.rs`, `features.rs`, `nn.rs`, `train.rs`, `play.rs`, `main.rs`) | Move generation identical to Python on about 10k positions (`tests/test_rust_movegen.py`) |
| JavaScript engine and network (browser) | `web/engine.js`, `web/net.js`, `web/index.html` | Identical move generation to Rust on 5,321 positions; network equity difference ≤ 5e-5; headless-browser full-game test (`web/tests/`) |
| Trained net | `engine-rs/nets/td-best.net` (2M games) | see §6 |

Rust subcommands: `train`, `bench`, `opening`, `replies`, `pinvalue`, `stats`, `releases`, `liftdata`, `quiz`, `compare`, `evalset`, `motherstats`, `choose`, `movegen`. Rule players are specified as `rule:<name>:<net>` in `bench`.

## 6. The neural network

- **Inputs (253):** for each side, 24 points × 4 TD-Gammon-style units (≥1, ≥2, ≥3, (n−3)/12), borne-off/15, and per-point pin risk (1 if pinned; otherwise the probability out of 36 that the opponent pins that blot next roll, Palamedes's key feature). Also: mother-exposed flag, checkers stacked on home pins, checkers below the deepest home pin (≈ ChFrontOfPin), pin count/5. Global: pip lead, pip deficit, race flag.
- **Network:** 253 → 100 sigmoid → 3 sigmoid (W, WD, LD); equity = 2W − 1 + WD − LD.
- **Training:** online TD(0) (update after every move toward the next position's value, inverted for side to move), greedy 1-ply self-play, **Hogwild with 4 threads** (lock-free shared weights). Learning rate 0.1 up to 1M games, 0.03 for 1–2M, 0.01 for 2–3M. Weights initialised uniform ±0.1. About 100 games/s.
  - This differs from Palamedes, who used reverse offline updates with target recalculation and initialised weights at ±0.5.
- **Strength:**
  - Against the heuristic: 50k games 71% → 1M 78–80% → 2M **81.5% (+1.06 ppg)**; random: 100%.
  - Head-to-head: 1M beats 50k by +0.32, 500k by +0.11; 2M beats 1M by **+0.061 (10k games, 52.2%)**; 3M vs 2M **+0.006 (20k games, 50.1% ±0.7)**. It has plateaued.
- **External check:** opening rollouts (4,000 games per candidate, top 3 per roll, net on both sides at 1-ply) match Palamedes's best move on **19 of 21 rolls**; the 2 misses (3-1, 2-2) are near-ties. First-player equity is +0.110 (Palamedes +0.104).
  - This is the **only** external validation of the bot's strength.

## 7. Findings the guide relies on

Unless noted: 2M net, 1-ply move choice, rollouts with the net on both sides, and equity in points per game (ppg). The standard error is roughly 1.45/√N (e.g. ±0.015 at 10k games, ±0.03 at 2k).

| Finding | Method | Numbers |
|---|---|---|
| Opening book | rollouts, top 3 per roll, 4k games each | 19/21 agree with Palamedes; always split two off the start on non-doubles |
| Second-player replies | **1-ply net only** (no rollouts) for 21×21 combinations | 302/315 non-double replies split; exceptions: run to unreachable points; punish builders on their 19/21 with 6-6/5-5 (about +0.9) |
| Pin value by point | synthetic: start position plus one pin on point p, pinner on roll, 4k rollouts | +1.6 (pt 2), +1.3 (3), +1.1 (4), about +1.0 (5–13), +0.5–0.8 (14–23); a sign bug in the raw output was corrected in `pin_value_clean.tsv` |
| Pin statistics in play | 20k self-play games | pin on 2: pinner wins 93% (83% mars); on 3: 81%; on 4: 69% |
| Risk profile | 20k self-play games | worst blot about 2.5/36 in turns 1–5, up to 6.7/36 in turns 16–20 |
| Unstacking speed | 20k self-play games | start emptied by own turn 13 (median); mother-pin endings 4–5% |
| Lifting pins | 134k keep-or-lift decisions from 8k games, **1-ply equities** | lifting right 3%; "never lift" agrees 97% (avg loss 0.004/decision); trade a 5/6-pin for a deeper 2–4 pin: right 41%; never lift when the pinned checker is the opponent's last straggler (0.3%) or the pin is on 2–3 (0.4%) |
| Counter-pins | 6k self-play games, **correlational** | deep-pinned players win 57% with a counter-pin vs 25% without |
| Six "costly mistakes" | positions where the heuristic's choice was worse than the net's (`quiz`), chosen by hand, each verified with 1,000–1,500 rollouts per candidate | e.g. take the deep pin (+0.5–1.1), don't lift the pin (1.4), clear the last two start checkers together (+0.25), counter-pin when pinned (+2.3) |
| Rules of thumb as strict policies | net constrained to rule-obeying moves vs plain net, 10k games each | never lift −0.02; never leave a lone mother −0.03; always take a home pin −0.06; always pin −0.15; never leave a 6+-shot blot −0.34; never a 12+-shot blot −0.33; never add to 1–2 before bear-off (pins allowed) −0.29; while pinned don't bring checkers home −0.32; maximise points −0.85 |
| Lone mother cost | 8 runner distances × {0, 1, 2 checkers on start}, 3k rollouts each | ≈ 0.07 ppg per pinning roll; leaving 2 is never worse than leaving 0 |
| Pin standoff | 3 depth combinations × 5×5 stack sizes, 2k rollouts each | one point of depth ≈ 1 ppg (beats a 6-checker stack advantage); +0.13 per extra stacked checker at equal depth; moving first ≈ −0.1 |
| Stacking on the pin in the bear-off | same pips, stacked vs spread, 8k rollouts | +0.10 (pin on 3), +0.29 (pin on 4) |
| Skill vs luck | per-game outcome rates → exact set probabilities | bot vs heuristic: 95% of sets to 3; near-equal (+0.06 ppg): 53% |

**Corrections already made** after testing:
- "Don't bury checkers on 1–2" was narrowed to a position-specific lesson: those points wall the opponent's start.
- "Don't crash your position when pinned" was rejected.

## 8. Assumptions and weaknesses a reviewer should check

1. **A single self-trained net is the oracle for nearly every claim.** Its only external check is the opening book. Mid-game and bear-off judgements aren't independently validated, so systematic blind spots of the net would pass straight into the guide.
2. **1-ply everywhere.** Move choice inside rollouts, the replies table and the lift analysis all use 1-ply. Palamedes measured +0.07 ppg from 2-ply. No variance reduction and no truncated rollouts were used.
3. **The rule-player test is relative to the same net.** "Cost of rule X" = net-with-constraint vs net, so it measures distance from the net's policy, and an imperfect net can't show a rule that *beats* it. The opponent is the net, not a human, so a rule that's costly against a strong player could still help a human against humans.
4. **The "mistakes" baseline is our own heuristic**, not real human play, and the examples were picked by hand. The themes may reflect the heuristic's quirks; for example, it doesn't value bearing off.
5. **Synthetic positions** (pin value, mother, standoff, stacking) are hand-built and may not be typical. The pin-value setup keeps the start position with one pin and the pinner on roll.
6. **Correlational self-play statistics** (counter-pin 57% vs 25%, pin win rates by point) are confounded with general position strength.
7. **Training choices differ from the published recipe:** online TD(0) instead of reverse-offline with target recalculation, Hogwild races, small initial weights, no feature ablation. The plateau at 2M may reflect capacity or features rather than a true limit. No larger net or richer features were tried.
8. **Money play vs match play:** the net maximises points per game, but Egyptian sets are to 3 points. The guide's set advice uses Palamedes's Plakoto match equity table plus reasoning, not a match-aware bot.
9. **No endgame databases** (Palamedes built pin-endgame and bear-off databases).
10. **Statistical rigour is light:** standard errors are reported but there's no multiple-comparison control, and some "ties" rest on differences of 1–2 SE.
11. **Rules risk:** the opening re-roll and the strict mother rule are decided. Regional variation in Egypt wasn't surveyed beyond the sources above.

## 9. Candidate next steps (unprioritised)

- **External validation:** compare a sample of mid-game and bear-off positions against Palamedes / Polygammon (free Windows app), or against strong human players.
- **Stronger oracle:** 2-ply (or 3-ply with pruning) for analysis; variance-reduced or truncated rollouts; a bigger net or extra features (escape probability, prime features) trained from scratch; the reverse-offline TD variant; pin-endgame databases.
- **Better rule tests:** run the rule players against a weaker (human-like) opponent as well as the net; test rule *combinations* as a full "human strategy"; report confidence intervals in the guide.
- **Match-aware play** for sets to 3 (match equity table in move choice) to check the guide's set advice.
- **Human-facing work:** a quiz or trainer from `analysis/net2m/quiz_candidates.tsv`; check that the guide is usable at the table (length, ordering, Arabic terms).

## 10. Repository map and reproduction

```
guide/STRATEGY_GUIDE.md      the guide (v2.2), every claim tagged by evidence
research/RESEARCH.md         rules research and sources
src/mahbusa/, tests/         Python engine, heuristic bot, terminal play (pytest)
engine-rs/                   Rust engine, trainer, analysis commands; run_training.sh (resumable),
                             run_analysis.sh, run_rule_tests.sh, run_position_tests.sh; nets/td-best.net
web/                         browser board (open web/index.html), coach, JS engine
analysis/                    1M-net results (top level), net2m/ (2M net, used by the guide),
                             pins/ (lift decisions, gzipped), experiments/ (rule tests, controlled positions)
scripts/                     shot table, quiz rendering/categorising, experiment builders, summaries
PROGRESS.md                  chronological work log
```

Setup:

```bash
python3 -m venv .venv
.venv/bin/pip install -e ".[dev]"
.venv/bin/pytest
```

Rust engine:

```bash
cd engine-rs && cargo build --release
```

Example: test a rule of thumb as a policy.

```bash
./target/release/mahbusa-rs bench --a rule:nolift:nets/td-best.net --b nets/td-best.net --games 10000
```

Operational notes:
- Cloud containers restart while idle, so long runs must be resumable. `run_training.sh` resumes from `nets/td-latest.net`.
- Commits are authored by the owner's account.
