"""Render quiz positions: python scripts/show_quiz.py analysis/net1m/quiz_candidates.tsv ROW [ROW...]"""
import csv
import sys

from mahbusa.render import render
from mahbusa.state import State

rows = list(csv.DictReader(open(sys.argv[1]), delimiter="\t"))
for i in map(int, sys.argv[2:]):
    r = rows[i]
    s = State(tuple(map(int, r["me"].split(","))), tuple(map(int, r["op"].split(","))), int(r["mpin"]), int(r["opin"]), 0)
    print(f"--- row {i}: game {r['game']} turn {r['turn']}  X to play {r['d1']}-{r['d2']}")
    print(render(s))
    print(f"best: {r['best']} ({r['best_eq']})   heuristic: {r['heur']} ({r['heur_eq']})   loss {r['loss']}\n")
