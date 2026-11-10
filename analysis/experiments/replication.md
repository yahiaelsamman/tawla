# Replication with an independent network

The guide's numbers come from one network (`engine-rs/nets/td-best.net`, 2M self-play games). To see whether they are an accident of that one training run, we trained a second network from scratch (`engine-rs/nets/td-alt.net`) and repeated the main experiments with it. Raw output is in `alt/`; the side-by-side numbers are in `replication_numbers.txt` (made by `scripts/summarize_replication.py`; the run is `engine-rs/run_replication.sh`).

**What the second network is.** Same architecture, input features, training code and recipe as the first (253 inputs, 100 hidden units, TD(0) self-play, Hogwild), but a different random seed and initial weights, and only 1M games (learning rate 0.1 up to 800k games, then 0.03). It scores 81.1% against the rule-based bot (the 2M net: 81.5%) and loses to the 2M net by 0.047 ± 0.021 points per game, about the same gap as the earlier 1M net had (0.061). So it is a slightly weaker, separately trained copy.

**What this does and doesn't show.** It shows the findings are not noise or a quirk of one training run. It doesn't show they are independent of the method: both nets share the same features, architecture and training recipe, so a blind spot built into those would be shared. Checking against an outside program (Palamedes) is still open.

| Finding (guide section) | 2M net | Independent net | Holds? |
|---|---|---|---|
| Opening: best 1-ply first move (§3) | | same first move for 21 of 21 rolls | yes |
| Pin value by point (§6), pinner's gain: pin on 2 / 3 / 4 / 6 / 13 / 23 | +1.64 / +1.28 / +1.09 / +1.01 / +0.94 / +0.59 | +1.64 / +1.34 / +1.14 / +1.09 / +0.99 / +0.66 | yes, same shape, within about 0.1 |
| Lone mother (§5): cost with a runner 2 / 6 / 8 / 9 away | −0.85 / −1.07 / −0.10 / −0.19 | −0.83 / −1.04 / −0.12 / −0.18 | yes |
| Standoff (§6): equal depth, 7 v 1 stack | +0.71 | +0.66 | yes |
| Standoff: you are deeper (3 v 4), equal stacks | +0.79 | +0.82 | yes |
| Standoff: they are deeper (4 v 3), equal stacks | −1.00 | −0.96 | yes |
| Standoff: they are deeper, you 7 v 1 | −0.54 | −0.48 | yes |
| Stacking in the bear-off (§7): pin on 3 / pin on 4 | +0.10 / +0.29 | +0.13 / +0.29 | yes |
| Rule costs (§12), expensive rules: 6+-shot blot / 12+-shot blot / no 1–2 points / no crash when pinned / maximise points | −0.34 / −0.33 / −0.29 / −0.32 / −0.85 | −0.35 / −0.33 / −0.28 / −0.33 / −0.87 | yes, all within 0.03 |
| Rule costs, always pin | −0.15 | −0.11 | yes (±0.03 at 10,000 games) |
| Rule costs, cheap rules: never lift / never lone start checker / always home pin | −0.02 / −0.03 / −0.06 | −0.04 / +0.02 / −0.04 | consistent with a small cost (all within ±0.03 noise at 10,000 games; the 100,000-game figures for the 2M net are −0.027 / −0.019 / −0.045) |
| Self-play: double-win rate, mother-pin endings, median turn the start empties (§1, §5) | 42.8% / 4.4% / 13 | 43.4% / 3.2% / 13 | yes |
| Example positions (§11): best move by playout in all eight Mistake positions | | same best move in 8 of 8 | yes |
| Trade-up examples (§7) | 24/19 5/2 and 22/16 5/2 | 24/19 5/2 and 24/18 5/2 | yes: both lift to pin on the 2-point; the two second-example moves are near-equal |

Nothing in the guide changed because of this replication. The one open difference is among the cheap rules, whose individual costs are too small to rank at 10,000 games; the guide uses the 100,000-game figures from the 2M net.
