# Experiments (guide v2.2, §5–§9 and §12)

All experiments use the 2M-game net (`engine-rs/nets/td-best.net`) unless a file says otherwise.

| File | Experiment | How to reproduce |
|---|---|---|
| `rules.txt` | Rules of thumb as strict policies: the net constrained to rule-obeying moves vs the plain net, 10,000 games each | `engine-rs/run_rule_tests.sh 10000` (rule names in `engine-rs/src/play.rs`, `obeys()`) |
| `mother.in/.tsv`, `mother_summary.txt` | Cost of a lone mother vs two or none, by runner distance | `scripts/experiments.py build`, then `engine-rs/run_position_tests.sh` |
| `release_race.in/.tsv`, `release_race_summary.txt` | Mutual-pin standoffs: pin depth × checkers stacked | same |
| `bearoff_stack.in/.tsv` | Bearing off with a pin: stacked on the pin vs spread (same pips) | same |
| `mother_rule.txt` | How often Egyptian vs Greek mother rules decide a game differently | `mahbusa-rs motherstats --net nets/td-best.net --games 20000` |
| `skill_vs_luck.txt` | Set-win chances from measured per-game results | computed from `bench` results |

`evalset` reports equity for the player on roll. In `mother.tsv` the opponent is on roll, so the summary flips the sign.

## Added in v2.3

| File | Experiment | How to reproduce |
|---|---|---|
| `rules_100k.txt` | The five cheap rules (never lift, never lift except trade-up, never a lone start checker, always home pin, always pin) at 100,000 games each, seed 42 | `SEED=42 OUT=../analysis/experiments/rules_100k.txt RULES="nolift nolift_tradeup pairs homepin alwayspin" engine-rs/run_rule_tests.sh 100000` |
| `rules_weak_player.txt` | The same rules applied to the simple rule-based bot, against its unrestricted self (20,000 games each) | `engine-rs/run_weak_player_tests.sh 20000` |
| `verify/`, `verify_summary.txt` | One-move judgement vs playouts: 300 keep-or-lift decisions and 100 quiz positions, 1,000 playouts per move | `scripts/sample_for_verify.py analysis/experiments/verify`, then `engine-rs/run_verify.sh`, then `scripts/summarize_verify.py analysis/experiments/verify` |
| `replies_rollout_sample.tsv` | Second-player replies chosen by playouts instead of one-move value, for four first rolls | `mahbusa-rs replies --net nets/td-best.net --first 31,62,65,55 --games 2000 --top 3` |

Note on `verify`: the first draw of the trade-up group was biased by a sampling mistake (the "big gain" group had already taken the cases where lifting is better from the same part of the shuffled list). The group was redrawn from the full population with its own generator; the other groups were unchanged, and `sample_for_verify.py` reproduces the final sample.
