"""Play with the trained neural network (runs the Rust engine as a helper process)."""
from __future__ import annotations

import subprocess
from pathlib import Path

from .agents import Agent

ROOT = Path(__file__).resolve().parents[2] / "engine-rs"
BINARIES = [ROOT / "target" / "release" / "mahbusa-rs", ROOT / "target-analysis" / "release" / "mahbusa-rs"]
DEFAULT_NETS = [ROOT / "nets" / "td-best.net", ROOT / "nets" / "td-latest.net"]


def find_binary() -> Path:
    for b in BINARIES:
        if b.exists():
            return b
    raise FileNotFoundError("build the Rust engine first: cd engine-rs && cargo build --release")


def find_net() -> Path:
    for n in DEFAULT_NETS:
        if n.exists():
            return n
    raise FileNotFoundError("no trained net found in engine-rs/nets/")


class NetAgent(Agent):
    """Greedy 1-ply neural-network player."""

    name = "net"

    def __init__(self, net_path: str | Path | None = None):
        net = Path(net_path) if net_path else find_net()
        self.proc = subprocess.Popen(
            [str(find_binary()), "choose", "--net", str(net)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1,
        )
        self.last_equity = None

    def choose(self, state, moves, dice, rng):
        line = f"{','.join(map(str, state.me))};{','.join(map(str, state.op))};{state.mpin};{state.opin};{dice[0]};{dice[1]}\n"
        self.proc.stdin.write(line)
        self.proc.stdin.flush()
        reply = self.proc.stdout.readline().strip()
        me_s, mpin_s, eq = reply.split("|")
        me = tuple(int(x) for x in me_s.split(","))
        mpin = int(mpin_s)
        self.last_equity = float(eq)
        for m in moves:
            if m.after.me == me and m.after.mpin == mpin:
                return m
        raise RuntimeError(f"engine chose a move the Python engine does not know: {reply}")

    def close(self):
        if self.proc.poll() is None:
            self.proc.stdin.close()
            self.proc.wait(timeout=5)
