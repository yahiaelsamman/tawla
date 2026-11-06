//! Board state and move generation (port of src/mahbusa/state.py, Egyptian rules).
//!
//! The state is always seen from the player to move ("me"). `me[p]` / `op[q]` are each
//! side's checkers in its OWN numbering (start = 24, home = 1..6, index 0 = borne off).
//! My point p is the opponent's point 25 - p. `mpin` bit p: I pin a lone opponent
//! checker on my point p. `opin` bit q: the opponent pins my checker on their point q.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

pub const START: usize = 24;
pub const N: u8 = 15;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct State {
    pub me: [u8; 25],
    pub op: [u8; 25],
    pub mpin: u32,
    pub opin: u32,
}

impl State {
    pub fn initial() -> Self {
        let mut s = [0u8; 25];
        s[START] = N;
        State { me: s, op: s, mpin: 0, opin: 0 }
    }
    #[inline]
    pub fn flip(&self) -> Self {
        State { me: self.op, op: self.me, mpin: self.opin, opin: self.mpin }
    }
    #[inline]
    pub fn opp_at(&self, p: usize) -> u8 {
        self.op[25 - p]
    }
    #[inline]
    pub fn my_pinned_at(&self, p: usize) -> bool {
        (self.opin >> (25 - p)) & 1 == 1
    }
    #[inline]
    pub fn i_pin_at(&self, p: usize) -> bool {
        (self.mpin >> p) & 1 == 1
    }
    pub fn pips(&self) -> (u32, u32) {
        let a = (1..25).map(|p| p as u32 * self.me[p] as u32).sum();
        let b = (1..25).map(|p| p as u32 * self.op[p] as u32).sum();
        (a, b)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub after: State, // mover's point of view, not flipped
    pub outcome: i8,  // 0 = game goes on, 1 = single win, 2 = double win (mars / mother)
    pub steps: [(u8, u8); 4],
    pub nsteps: u8,
}

impl Move {
    pub fn notation(&self) -> String {
        let v: Vec<String> = self.steps[..self.nsteps as usize]
            .iter()
            .map(|&(a, b)| if b == 0 { format!("{}/off", a) } else { format!("{}/{}", a, b) })
            .collect();
        if v.is_empty() { "(no move)".into() } else { v.join(" ") }
    }
}

#[inline]
fn can_land(op: &[u8; 25], opin: u32, q: usize) -> bool {
    let oc = op[25 - q];
    oc == 0 || (oc == 1 && (opin >> (25 - q)) & 1 == 0)
}

#[inline]
fn all_home(me: &[u8; 25]) -> bool {
    me[7..25].iter().all(|&c| c == 0)
}

#[inline]
fn highest(me: &[u8; 25]) -> usize {
    (1..25).rev().find(|&p| me[p] > 0).unwrap_or(0)
}

/// Move one of my checkers from p by d. Returns (new state, to, outcome) or None.
#[inline]
pub fn step(s: &State, p: usize, d: usize) -> Option<(State, u8, i8)> {
    if s.me[p] == 0 || (s.opin >> (25 - p)) & 1 == 1 {
        return None;
    }
    let q = p as i32 - d as i32;
    if q >= 1 {
        if !can_land(&s.op, s.opin, q as usize) {
            return None;
        }
    } else {
        if s.opin != 0 || !all_home(&s.me) {
            return None;
        }
        if q < 0 && highest(&s.me) != p {
            return None;
        }
    }
    let mut n = *s;
    n.me[p] -= 1;
    if n.me[p] == 0 && (n.mpin >> p) & 1 == 1 {
        n.mpin &= !(1u32 << p);
    }
    let mut outcome = 0i8;
    let to;
    if q >= 1 {
        let q = q as usize;
        n.me[q] += 1;
        if n.op[25 - q] == 1 && (n.mpin >> q) & 1 == 0 {
            n.mpin |= 1u32 << q;
            if q == 1 {
                outcome = 2; // mother pinned: immediate double win (Egyptian rule)
            }
        }
        to = q as u8;
    } else {
        n.me[0] += 1;
        to = 0;
        if n.me[0] == N {
            outcome = if n.op[0] == 0 { 2 } else { 1 };
        }
    }
    Some((n, to, outcome))
}

// ---------------------------------------------------------------- fast hashing
#[derive(Default)]
pub struct FxHasher(u64);
impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut v = 0u64;
            for (i, b) in chunk.iter().enumerate() {
                v |= (*b as u64) << (8 * i);
            }
            self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
        }
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0 = (self.0.rotate_left(5) ^ i as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.0 = (self.0.rotate_left(5) ^ i as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}
type Fx = BuildHasherDefault<FxHasher>;

#[derive(Clone, Copy)]
struct Leaf {
    used: u8,
    first_die: u8,
    mv: Move,
}

/// Reusable move generator (keeps its hash tables between calls).
pub struct MoveGen {
    seen: HashSet<([u8; 25], u32, u8), Fx>,
    leaves: HashMap<([u8; 25], u32), Leaf, Fx>,
}

impl Default for MoveGen {
    fn default() -> Self {
        MoveGen { seen: HashSet::default(), leaves: HashMap::default() }
    }
}

impl MoveGen {
    /// All distinct legal moves for (d1, d2); empty = pass.
    pub fn legal_moves(&mut self, s: &State, d1: u8, d2: u8, out: &mut Vec<Move>) {
        out.clear();
        self.seen.clear();
        self.leaves.clear();
        let doubles = d1 == d2;
        let n_dice: u8 = if doubles { 4 } else { 2 };
        let orders: Vec<[u8; 4]> = if doubles {
            vec![[d1; 4]]
        } else {
            vec![[d1, d2, 0, 0], [d2, d1, 0, 0]]
        };
        for (oi, dice) in orders.iter().enumerate() {
            let empty = Move { after: *s, outcome: 0, steps: [(0, 0); 4], nsteps: 0 };
            self.rec(s, dice, n_dice, 0, oi as u8, empty);
        }
        if self.leaves.is_empty() {
            return;
        }
        let max_used = self.leaves.values().map(|l| l.used).max().unwrap();
        if max_used == 0 {
            return;
        }
        let big = d1.max(d2);
        let need_big = max_used == 1
            && !doubles
            && self.leaves.values().any(|l| l.used == 1 && l.first_die == big);
        for l in self.leaves.values() {
            if l.used != max_used {
                continue;
            }
            if need_big && l.first_die != big {
                continue;
            }
            out.push(l.mv);
        }
        // deterministic order: sort by resulting position
        out.sort_by(|a, b| (a.after.me, a.after.mpin).cmp(&(b.after.me, b.after.mpin)));
    }

    fn rec(&mut self, s: &State, dice: &[u8; 4], n_dice: u8, idx: u8, order: u8, mv: Move) {
        let rem_sig = if n_dice == 4 { 4 - idx } else { order * 4 + idx };
        if !self.seen.insert((s.me, s.mpin, rem_sig)) {
            return;
        }
        let mut moved = false;
        if idx < n_dice {
            let d = dice[idx as usize] as usize;
            for p in (1..25).rev() {
                if s.me[p] == 0 {
                    continue;
                }
                if let Some((n, to, outcome)) = step(s, p, d) {
                    moved = true;
                    let mut m = mv;
                    m.steps[idx as usize] = (p as u8, to);
                    m.nsteps = idx + 1;
                    m.after = n;
                    if outcome != 0 {
                        m.outcome = outcome;
                        self.record(n_dice, dice[0], m);
                    } else {
                        self.rec(&n, dice, n_dice, idx + 1, order, m);
                    }
                }
            }
        }
        if !moved {
            let first = if idx > 0 { dice[0] } else { 0 };
            let mut m = mv;
            m.after = *s;
            self.record(idx, first, m);
        }
    }

    fn record(&mut self, used: u8, first_die: u8, mv: Move) {
        let k = (mv.after.me, mv.after.mpin);
        let new = Leaf { used, first_die, mv };
        match self.leaves.get(&k) {
            Some(prev) if (prev.used, prev.first_die) >= (used, first_die) => {}
            _ => {
                self.leaves.insert(k, new);
            }
        }
    }
}
