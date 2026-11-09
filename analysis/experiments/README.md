# Experiments (guide v2.2, §5–§9 and §12)

All experiments use the 2M-game net (`engine-rs/nets/td-best.net`).

| File | Experiment | How to reproduce |
|---|---|---|
| `rules.txt` | Rules of thumb as strict policies: the net constrained to rule-obeying moves vs the plain net, 10,000 games each | `engine-rs/run_rule_tests.sh 10000` (rule names in `engine-rs/src/play.rs`, `obeys()`) |
| `mother.in/.tsv`, `mother_summary.txt` | Cost of a lone mother vs two or none, by runner distance | `scripts/experiments.py build`, then `engine-rs/run_position_tests.sh` |
| `release_race.in/.tsv`, `release_race_summary.txt` | Mutual-pin standoffs: pin depth × checkers stacked | same |
| `bearoff_stack.in/.tsv` | Bearing off with a pin: stacked on the pin vs spread (same pips) | same |
| `mother_rule.txt` | How often Egyptian vs Greek mother rules decide a game differently | `mahbusa-rs motherstats --net nets/td-best.net --games 20000` |
| `skill_vs_luck.txt` | Set-win chances from measured per-game results | computed from `bench` results |

`evalset` reports equity for the player on roll. In `mother.tsv` the opponent is on roll, so the summary flips the sign.
