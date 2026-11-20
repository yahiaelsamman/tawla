// node web/tests/check_ply2.js cases.txt rust_bin net_file [N]
// The browser engine's 2-ply values must match the Rust engine's (`compare --games 0 --ply2 1`).
const fs = require("fs");
const { execFileSync } = require("child_process");
const M = require("../engine.js");
const [cases, bin, netPath, nArg] = process.argv.slice(2);
const net = M.loadNet(new Uint8Array(fs.readFileSync(netPath)));
const lines = fs.readFileSync(cases, "utf8").trim().split("\n");
const N = Math.min(+(nArg || 150), lines.length);
let checked = 0, bad = 0, maxDiff = 0;
for (let i = 0; i < lines.length && checked < N; i += Math.max(1, Math.floor(lines.length / N))) {
  const f = lines[i].split(";");
  const s = { me: f[0].split(",").map(Number), op: f[1].split(",").map(Number), mpin: +f[2], opin: +f[3] };
  const moves = M.legalMoves(s, +f[4], +f[5]);
  if (moves.length < 2 || moves.some((m) => m.outcome > 0)) continue;
  const out = execFileSync(bin, ["compare", "--net", netPath, "--pos", f.slice(0, 4).join(";"), "--dice", f[4] + f[5], "--games", "0", "--top", "3", "--ply2", "1"], { encoding: "utf8" });
  const rs = out.trim().split("\n").map((l) => +l.match(/ply2 ([+-][\d.]+)/)[1]).sort((a, b) => b - a);
  const js = M.rankMoves(net, moves).slice(0, 3).map((r) => M.moveValue2(net, r.move)).sort((a, b) => b - a);
  const d = Math.max(...rs.map((v, k) => Math.abs(v - js[k])));
  maxDiff = Math.max(maxDiff, d);
  if (d > 2e-3) { bad++; if (bad < 4) console.log("MISMATCH", lines[i], rs, js); }
  checked++;
}
console.log(`${checked} positions: 2-ply mismatches ${bad}, max difference ${maxDiff.toExponential(2)}`);
process.exit(bad ? 1 : 0);
