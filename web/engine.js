/* Mahbusa engine + neural-network player in plain JavaScript.
 * A line-for-line port of engine-rs (state.rs, features.rs, nn.rs); Egyptian rules.
 * State is from the player to move: me[p] / op[q] in each side's own numbering
 * (start = 24, home = 1..6, index 0 = borne off); my point p is the opponent's 25 - p.
 * mpin bit p: I pin a lone opponent checker on my point p.
 * opin bit q: the opponent pins my checker on their point q.
 * Works in the browser (window.Mahbusa) and in Node (module.exports). */
(function (root) {
  "use strict";
  const START = 24, N = 15;

  function initial() {
    const s = new Array(25).fill(0);
    s[START] = N;
    return { me: s.slice(), op: s.slice(), mpin: 0, opin: 0 };
  }
  function flip(s) {
    return { me: s.op, op: s.me, mpin: s.opin, opin: s.mpin };
  }
  const myPinnedAt = (s, p) => ((s.opin >>> (25 - p)) & 1) === 1;
  const iPinAt = (s, p) => ((s.mpin >>> p) & 1) === 1;
  function pips(s) {
    let a = 0, b = 0;
    for (let p = 1; p < 25; p++) { a += p * s.me[p]; b += p * s.op[p]; }
    return [a, b];
  }
  function canLand(op, opin, q) {
    const oc = op[25 - q];
    return oc === 0 || (oc === 1 && ((opin >>> (25 - q)) & 1) === 0);
  }
  function allHome(me) {
    for (let p = 7; p < 25; p++) if (me[p]) return false;
    return true;
  }
  function highest(me) {
    for (let p = 24; p >= 1; p--) if (me[p]) return p;
    return 0;
  }

  /** Move one of my checkers from p by d. Returns {state, to, outcome} or null. */
  function step(s, p, d) {
    if (s.me[p] === 0 || ((s.opin >>> (25 - p)) & 1) === 1) return null;
    const q = p - d;
    if (q >= 1) {
      if (!canLand(s.op, s.opin, q)) return null;
    } else {
      if (s.opin !== 0 || !allHome(s.me)) return null;
      if (q < 0 && highest(s.me) !== p) return null;
    }
    const me = s.me.slice();
    let mpin = s.mpin;
    me[p] -= 1;
    if (me[p] === 0 && ((mpin >>> p) & 1) === 1) mpin &= ~(1 << p);
    let outcome = 0, to;
    if (q >= 1) {
      me[q] += 1;
      if (s.op[25 - q] === 1 && ((mpin >>> q) & 1) === 0) {
        mpin |= 1 << q;
        if (q === 1) outcome = 2; // mother pinned: immediate double win
      }
      to = q;
    } else {
      me[0] += 1;
      to = 0;
      if (me[0] === N) outcome = s.op[0] === 0 ? 2 : 1;
    }
    return { state: { me, op: s.op, mpin, opin: s.opin }, to, outcome };
  }

  const keyOf = (s) => s.me.join(",") + "|" + s.mpin;

  /** Every legal way to play the roll, as step sequences [[from, to, die], ...]
   * (all orders kept, for click-by-click play). Applies "use as many dice as
   * possible" and "play the larger die if only one can be used". [] = pass. */
  function legalPaths(s, d1, d2) {
    const orders = d1 === d2 ? [[d1, d1, d1, d1]] : [[d1, d2], [d2, d1]];
    const paths = [];
    function rec(st, dice, idx, steps) {
      let moved = false;
      if (idx < dice.length) {
        for (let p = 24; p >= 1; p--) {
          if (!st.me[p]) continue;
          const r = step(st, p, dice[idx]);
          if (!r) continue;
          moved = true;
          const ns = steps.concat([[p, r.to, dice[idx]]]);
          if (r.outcome) paths.push({ steps: ns, used: dice.length, first: dice[0], after: r.state, outcome: r.outcome });
          else rec(r.state, dice, idx + 1, ns);
        }
      }
      if (!moved) paths.push({ steps, used: idx, first: idx ? dice[0] : 0, after: st, outcome: 0 });
    }
    for (const dice of orders) rec(s, dice, 0, []);
    let maxUsed = 0;
    for (const p of paths) maxUsed = Math.max(maxUsed, p.used);
    if (maxUsed === 0) return [];
    let out = paths.filter((p) => p.used === maxUsed);
    if (maxUsed === 1 && d1 !== d2) {
      const big = Math.max(d1, d2);
      if (out.some((p) => p.first === big)) out = out.filter((p) => p.first === big);
    }
    return out;
  }

  /** Distinct legal moves (one representative path per resulting position). */
  function legalMoves(s, d1, d2) {
    const seen = new Map();
    for (const p of legalPaths(s, d1, d2)) {
      const k = keyOf(p.after) + "|" + p.outcome;
      if (!seen.has(k)) seen.set(k, p);
    }
    return [...seen.values()];
  }

  function notation(steps) {
    return steps.length ? steps.map(([a, b]) => `${a}/${b === 0 ? "off" : b}`).join(" ") : "(no move)";
  }

  // ------------------------------------------------------------- features
  const ROLLS = [];
  for (let a = 1; a <= 6; a++) for (let b = a; b <= 6; b++) ROLLS.push([a, b, a === b ? 1 : 2]);

  function canLandF(s, q) {
    if (q < 1) return false;
    const oc = s.op[25 - q];
    return oc === 0 || (oc === 1 && ((s.opin >>> (25 - q)) & 1) === 0);
  }
  function canReach(s, src, target, d1, d2) {
    const dist = src - target;
    if (dist <= 0) return false;
    if (d1 === d2) {
      if (dist % d1 !== 0 || dist / d1 > 4) return false;
      for (let k = 1; k <= dist / d1; k++) if (!canLandF(s, src - k * d1)) return false;
      return true;
    }
    if (dist === d1 || dist === d2) return canLandF(s, target);
    if (dist === d1 + d2) return canLandF(s, target) && (canLandF(s, src - d1) || canLandF(s, src - d2));
    return false;
  }
  /** Rolls out of 36 with which the player to move can land on their point `target`. */
  function pinShots(s, target) {
    const srcs = [];
    for (let p = target + 1; p < 25; p++) if (s.me[p] > 0 && !myPinnedAt(s, p)) srcs.push(p);
    if (!srcs.length) return 0;
    let total = 0;
    for (const [d1, d2, w] of ROLLS) if (srcs.some((p) => canReach(s, p, target, d1, d2))) total += w;
    return total;
  }
  const isMyBlot = (s, p) => s.me[p] === 1 && !iPinAt(s, p) && !myPinnedAt(s, p);

  const PER_SIDE = 24 * 4 + 1 + 24 + 4;
  const N_INPUTS = 2 * PER_SIDE + 3;

  function encodeSide(s, oppView, out, o) {
    let i = o;
    for (let p = 1; p < 25; p++) {
      const c = s.me[p];
      out[i] = c >= 1 ? 1 : 0;
      out[i + 1] = c >= 2 ? 1 : 0;
      out[i + 2] = c >= 3 ? 1 : 0;
      out[i + 3] = c > 3 ? (c - 3) / 12 : 0;
      i += 4;
    }
    out[i++] = s.me[0] / 15;
    for (let p = 1; p < 25; p++) {
      out[i++] = myPinnedAt(s, p) ? 1 : isMyBlot(s, p) ? pinShots(oppView, 25 - p) / 36 : 0;
    }
    out[i] = s.me[START] === 1 && !myPinnedAt(s, START) ? 1 : 0;
    let stacked = 0, deepest = 0;
    for (let p = 1; p < 7; p++) if (iPinAt(s, p)) { stacked += s.me[p]; deepest = Math.max(deepest, p); }
    let below = 0;
    if (deepest > 0) { for (let p = 1; p < deepest; p++) below += s.me[p]; below += s.me[0]; }
    out[i + 1] = stacked / 15;
    out[i + 2] = below / 15;
    let pins = 0;
    for (let p = 1; p < 25; p++) if (iPinAt(s, p)) pins++;
    out[i + 3] = pins / 5;
  }

  function encode(s, out) {
    const f = flip(s);
    encodeSide(s, f, out, 0);
    encodeSide(f, s, out, PER_SIDE);
    const [mp, opp] = pips(s);
    const i = 2 * PER_SIDE;
    const diff = opp - mp;
    out[i] = diff > 0 ? diff / 100 : 0;
    out[i + 1] = diff < 0 ? -diff / 100 : 0;
    let myHigh = 0, theirLow = 25;
    for (let p = 24; p >= 1; p--) if (s.me[p] > 0) { myHigh = p; break; }
    for (let p = 1; p < 25; p++) if (s.op[25 - p] > 0) { theirLow = p; break; }
    out[i + 2] = myHigh < theirLow ? 1 : 0;
  }

  // ------------------------------------------------------------- network
  function loadNet(bytes) {
    const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const magic = String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
    if (magic !== "MHBN") throw new Error("not a Mahbusa net");
    const nIn = Number(dv.getBigUint64(4, true));
    const nHid = Number(dv.getBigUint64(12, true));
    const games = Number(dv.getBigUint64(20, true));
    if (nIn !== N_INPUTS) throw new Error("input size mismatch");
    let pos = 28;
    const take = (n) => {
      const a = new Float32Array(n);
      for (let i = 0; i < n; i++) a[i] = dv.getFloat32(pos + 4 * i, true);
      pos += 4 * n;
      return a;
    };
    const w1 = take(nHid * nIn), b1 = take(nHid), w2 = take(3 * nHid), b2 = take(3);
    return { nIn, nHid, games, w1, b1, w2, b2, x: new Float64Array(nIn), h: new Float64Array(nHid) };
  }
  const sigmoid = (x) => 1 / (1 + Math.exp(-x));

  /** [W, WD, LD] for the player to move in s. */
  function evaluate(net, s) {
    const { nIn, nHid, w1, b1, w2, b2, x, h } = net;
    encode(s, x);
    for (let j = 0; j < nHid; j++) {
      let acc = b1[j];
      const o = j * nIn;
      for (let i = 0; i < nIn; i++) if (x[i] !== 0) acc += w1[o + i] * x[i];
      h[j] = sigmoid(acc);
    }
    const out = [0, 0, 0];
    for (let k = 0; k < 3; k++) {
      let acc = b2[k];
      for (let j = 0; j < nHid; j++) acc += w2[k * nHid + j] * h[j];
      out[k] = sigmoid(acc);
    }
    return out;
  }
  const equity = (o) => 2 * o[0] - 1 + o[1] - o[2];

  /** Expected points for the mover of a move (exact for game-ending moves). */
  function moveValue(net, m) {
    return m.outcome > 0 ? m.outcome : -equity(evaluate(net, flip(m.after)));
  }
  /** Rank distinct moves, best first: [{move, value}]. */
  function rankMoves(net, moves) {
    return moves
      .map((m) => ({ move: m, value: moveValue(net, m) }))
      .sort((a, b) => (b.move.outcome > 0 ? 10 * b.move.outcome : b.value) - (a.move.outcome > 0 ? 10 * a.move.outcome : a.value));
  }

  function b64ToBytes(b64) {
    if (typeof atob === "function") {
      const bin = atob(b64);
      const u = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) u[i] = bin.charCodeAt(i);
      return u;
    }
    return new Uint8Array(Buffer.from(b64, "base64"));
  }

  const api = {
    START, N, initial, flip, step, legalPaths, legalMoves, notation, pips, keyOf,
    myPinnedAt, iPinAt, pinShots, encode, N_INPUTS, loadNet, evaluate, equity, moveValue, rankMoves, b64ToBytes,
  };
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  else root.Mahbusa = api;
})(typeof window !== "undefined" ? window : globalThis);
