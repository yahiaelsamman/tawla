// node web/tests/check_engine.js cases.txt rust_movegen.txt rust_choose.txt net_file
const fs = require("fs");
const M = require("../engine.js");
const [cases, rsMoves, rsChoose, netPath] = process.argv.slice(2);
const lines = fs.readFileSync(cases, "utf8").trim().split("\n");
const rm = fs.readFileSync(rsMoves, "utf8").split("\n");
const rc = fs.readFileSync(rsChoose, "utf8").split("\n");
const net = M.loadNet(new Uint8Array(fs.readFileSync(netPath)));
let bad = 0, badChoice = 0, maxEqDiff = 0;
lines.forEach((line, i) => {
  const f = line.split(";");
  const s = { me: f[0].split(",").map(Number), op: f[1].split(",").map(Number), mpin: +f[2], opin: +f[3] };
  const moves = M.legalMoves(s, +f[4], +f[5]);
  const js = moves.map((m) => `${m.after.me.join(",")}|${m.after.mpin}|${m.outcome}`).sort().join(" ");
  const rs = rm[i].trim().split(" ").filter(Boolean).sort().join(" ");
  if (js !== rs) { bad++; if (bad < 4) console.log("MOVEGEN MISMATCH", line, "\n js:", js, "\n rs:", rs); }
  if (!moves.length) { if (rc[i].trim() !== "pass") badChoice++; return; }
  const best = M.rankMoves(net, moves)[0];
  const [rme, rmp, req] = rc[i].trim().split("|");
  const eqd = Math.abs(best.value - +req);
  maxEqDiff = Math.max(maxEqDiff, eqd);
  if (`${best.move.after.me.join(",")}|${best.move.after.mpin}` !== `${rme}|${rmp}` && eqd > 1e-3) {
    badChoice++;
    if (badChoice < 4) console.log("CHOICE MISMATCH", line, best.value, req);
  }
});
console.log(`${lines.length} positions: movegen mismatches ${bad}, choice mismatches ${badChoice}, max equity diff ${maxEqDiff.toExponential(2)}`);
process.exit(bad || badChoice ? 1 : 0);
