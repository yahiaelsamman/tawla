# Analysis results

Produced by `engine-rs/run_analysis.sh <net>` and summarised with `scripts/summarize_analysis.py <dir>`.

- `net2m/`: the **2M-game net (td-best / td-final). The guide v2 uses these numbers.**
- Top level (`analysis/*.tsv`, `*.txt`): the earlier 1M-game net, kept for comparison.

| File | What it is |
|---|---|
| `opening_rollouts.tsv` | Top-3 first moves per roll, each rolled out 4,000 games (equity for the first player). |
| `opening_book.tsv` | Best move per roll compared with the Palamedes book. |
| `replies_1ply.tsv` | Best second-player reply for every (first roll, reply roll), using the net's 1-ply evaluation. |
| `pin_value.tsv` / `pin_value_clean.tsv` | Value of one early pin by point. In the raw file the equity is the pinned side's (the pinner is on roll); the clean file gives it from the pinner's side. |
| `selfplay_stats.txt` | 20,000 net-vs-net games: double-win rate, unstacking speed, pin statistics by point, risk profile. |
| `releases.txt` | When home-board pins are released (voluntary vs forced) and win rates of deep-pinned players with or without a counter-pin. |
| `quiz_candidates.tsv` | Positions where the heuristic bot's move is much worse than the net's (from `mahbusa-rs quiz`). |
| `quiz_verified.txt` | Rollout checks of the positions used in guide §11. |
