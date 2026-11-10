# Analysis results

Produced by `engine-rs/run_analysis.sh <net>` (writes to `analysis/net2m/` unless `OUT` is set) and summarised with `scripts/summarize_analysis.py <dir>`.

- `net2m/`: the **2M-game net (`engine-rs/nets/td-best.net`). The guide uses these numbers.**
- `net1m/`: the earlier 1M-game net, kept for comparison.
- `pins/`: the keep-or-lift decisions behind guide §7.
- `experiments/`: controlled-position experiments, rule tests, and the checks of the net's judgement (see the README there).

| File | What it is |
|---|---|
| `opening_rollouts.tsv` | Top-3 first moves per roll, each rolled out 4,000 games (equity for the first player). |
| `opening_book.tsv` | Best move per roll compared with the Palamedes book. |
| `replies_1ply.tsv` | Best second-player reply for every (first roll, reply roll), using the net's 1-ply evaluation. |
| `pin_value.tsv` / `pin_value_clean.tsv` | Value of one early pin by point. In the raw file the equity is the pinned side's (the pinner is on roll); the clean file gives it from the pinner's side. |
| `selfplay_stats.txt` | 20,000 net-vs-net games: double-win rate, unstacking speed, pin statistics by point, risk profile. |
| `releases.txt` | When home-board pins are released (voluntary vs forced) and win rates of deep-pinned players with or without a counter-pin. |
| `quiz_candidates.tsv` | Positions where the heuristic bot's move is much worse than the net's (from `mahbusa-rs quiz`). The row numbers in `net2m/quiz_verified.txt` refer to `net1m/quiz_candidates.tsv` (guide §11 Mistakes 1–4); those in `net2m/examples2_verified.txt` refer to `net2m/quiz_candidates.tsv` (Mistakes 5–6). (`scripts/replication_compare.py` lists them). |
| `quiz_verified.txt` | Rollout checks of the positions used in guide §11. |
