//! Players, games, benchmarks and rollouts.

use crate::features::{is_my_blot, pin_shots};
use crate::nn::{equity, Net, Scratch};
use crate::rng::Rng;
use crate::state::{Move, MoveGen, State, START};
use std::sync::Arc;

pub trait Player {
    fn choose(&mut self, s: &State, moves: &[Move], rng: &mut Rng) -> usize;
}

pub struct RandomPlayer;
impl Player for RandomPlayer {
    fn choose(&mut self, _s: &State, moves: &[Move], rng: &mut Rng) -> usize {
        rng.below(moves.len() as u64) as usize
    }
}

/// Port of the Python HeuristicAgent (same weights).
pub struct HeuristicPlayer;
impl HeuristicPlayer {
    pub fn evaluate(s: &State) -> f32 {
        let (mp, op) = s.pips();
        let mut score = 0.01 * (op as f32 - mp as f32);
        for p in 1..25 {
            if s.i_pin_at(p) {
                score += 1.0 + 0.25 * (12i32 - p as i32).max(0) as f32;
            } else if s.me[p] >= 2 {
                score += 0.25;
            }
            if s.my_pinned_at(p) {
                score -= 1.5 + 0.25 * (p as i32 - 13).max(0) as f32;
            }
        }
        let f = s.flip();
        for p in 1..25 {
            if is_my_blot(s, p) {
                let shots = pin_shots(&f, 25 - p) as f32;
                let danger = 1.0 + (p as i32 - 12).max(0) as f32 * 0.25;
                score -= 2.0 * danger * shots / 36.0;
            }
        }
        if s.me[START] == 1 && !s.my_pinned_at(START) {
            score -= 3.0 * pin_shots(&f, 1) as f32 / 36.0 * 10.0;
        }
        score
    }
}
impl Player for HeuristicPlayer {
    fn choose(&mut self, _s: &State, moves: &[Move], _rng: &mut Rng) -> usize {
        let mut best = (f32::NEG_INFINITY, 0);
        for (i, m) in moves.iter().enumerate() {
            let v = if m.outcome > 0 { 1000.0 * m.outcome as f32 } else { Self::evaluate(&m.after) };
            if v > best.0 {
                best = (v, i);
            }
        }
        best.1
    }
}

/// Greedy 1-ply neural-network player.
pub struct NetPlayer {
    pub net: Arc<Net>,
    sc: Scratch,
}
impl NetPlayer {
    pub fn new(net: Arc<Net>) -> Self {
        let sc = Scratch::new(&net);
        NetPlayer { net, sc }
    }
    /// Equity for the mover of making move m.
    pub fn move_value(&mut self, m: &Move) -> f32 {
        if m.outcome > 0 {
            m.outcome as f32
        } else {
            -equity(self.net.eval(&m.after.flip(), &mut self.sc))
        }
    }
}
impl Player for NetPlayer {
    fn choose(&mut self, _s: &State, moves: &[Move], _rng: &mut Rng) -> usize {
        let mut best = (f32::NEG_INFINITY, 0);
        for (i, m) in moves.iter().enumerate() {
            let v = if m.outcome > 0 { 10.0 * m.outcome as f32 } else { self.move_value(m) };
            if v > best.0 {
                best = (v, i);
            }
        }
        best.1
    }
}

pub struct GameOut {
    pub winner: usize, // absolute player index 0/1
    pub points: i8,
    pub mother: bool,
    pub turns: u32,
}

/// Play from `start` with `to_move` (absolute index) on roll. players[0], players[1].
pub fn play_from(
    start: State,
    to_move: usize,
    players: &mut [&mut dyn Player; 2],
    rng: &mut Rng,
    gen: &mut MoveGen,
) -> GameOut {
    let mut s = start;
    let mut cur = to_move;
    let mut moves = Vec::with_capacity(64);
    for turn in 1..5000u32 {
        let (d1, d2) = (rng.die(), rng.die());
        gen.legal_moves(&s, d1, d2, &mut moves);
        if moves.is_empty() {
            s = s.flip();
            cur = 1 - cur;
            continue;
        }
        let i = if moves.len() == 1 { 0 } else { players[cur].choose(&s, &moves, rng) };
        let m = moves[i];
        if m.outcome > 0 {
            return GameOut { winner: cur, points: m.outcome, mother: m.after.me[0] < 15, turns: turn };
        }
        s = m.after.flip();
        cur = 1 - cur;
    }
    panic!("game did not finish");
}

pub struct Tally {
    pub games: u64,
    pub wins: [u64; 2],
    pub points: [i64; 2],
    pub doubles: [u64; 2],
    pub mothers: u64,
    pub turns: u64,
}
impl Tally {
    pub fn new() -> Self {
        Tally { games: 0, wins: [0; 2], points: [0; 2], doubles: [0; 2], mothers: 0, turns: 0 }
    }
    pub fn add(&mut self, g: &GameOut) {
        self.games += 1;
        self.wins[g.winner] += 1;
        self.points[g.winner] += g.points as i64;
        if g.points == 2 {
            self.doubles[g.winner] += 1;
        }
        if g.mother {
            self.mothers += 1;
        }
        self.turns += g.turns as u64;
    }
    pub fn merge(&mut self, o: &Tally) {
        self.games += o.games;
        for i in 0..2 {
            self.wins[i] += o.wins[i];
            self.points[i] += o.points[i];
            self.doubles[i] += o.doubles[i];
        }
        self.mothers += o.mothers;
        self.turns += o.turns;
    }
    pub fn ppg(&self) -> f64 {
        (self.points[0] - self.points[1]) as f64 / self.games.max(1) as f64
    }
    pub fn summary(&self, a: &str, b: &str) -> String {
        let n = self.games.max(1) as f64;
        let w = self.wins[0] as f64 / n;
        // Each game is worth ±1 or ±2 points, so the mean square is (n + 3d) / n.
        let d = (self.doubles[0] + self.doubles[1]) as f64;
        let ppg = self.ppg();
        let ppg_ci = 1.96 * (((n + 3.0 * d) / n - ppg * ppg).max(0.0) / n).sqrt();
        format!(
            "{} games  {} vs {}: win {:.1}% ±{:.1}  ppg {:+.3} ±{:.3}  doubles {:.1}%/{:.1}%  mother-pin {:.1}%  avg turns {:.1}",
            self.games,
            a,
            b,
            100.0 * w,
            196.0 * (w * (1.0 - w) / n).sqrt(),
            ppg,
            ppg_ci,
            100.0 * self.doubles[0] as f64 / n,
            100.0 * self.doubles[1] as f64 / n,
            100.0 * self.mothers as f64 / n,
            self.turns as f64 / n
        )
    }
}

pub fn make_player(spec: &str) -> Box<dyn Player + Send> {
    if let Some(rest) = spec.strip_prefix("rule:") {
        // rule:<name>:<net path>
        let (rule, path) = rest.split_once(':').expect("rule:<name>:<net>");
        let net = Arc::new(Net::load(path).unwrap_or_else(|e| panic!("cannot load {}: {}", path, e)));
        return Box::new(RulePlayer::new(rule, net));
    }
    match spec {
        "random" => Box::new(RandomPlayer),
        "heuristic" => Box::new(HeuristicPlayer),
        path => {
            let net = Arc::new(Net::load(path).unwrap_or_else(|e| panic!("cannot load {}: {}", path, e)));
            Box::new(NetPlayer::new(net))
        }
    }
}

/// Play n games between spec a (player 0) and spec b, alternating starters, on `threads`.
pub fn bench(a: &str, b: &str, n: u64, seed: u64, threads: usize) -> Tally {
    let per = (n + threads as u64 - 1) / threads as u64;
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let (a, b) = (a.to_string(), b.to_string());
            std::thread::spawn(move || {
                let mut pa = make_player(&a);
                let mut pb = make_player(&b);
                let mut rng = Rng::new(seed.wrapping_mul(1000).wrapping_add(t as u64));
                let mut gen = MoveGen::default();
                let mut tally = Tally::new();
                for i in 0..per {
                    let mut ps: [&mut dyn Player; 2] = [pa.as_mut(), pb.as_mut()];
                    let g = play_from(State::initial(), (i % 2) as usize, &mut ps, &mut rng, &mut gen);
                    tally.add(&g);
                }
                tally
            })
        })
        .collect();
    let mut total = Tally::new();
    for h in handles {
        total.merge(&h.join().unwrap());
    }
    total
}


// ------------------------------------------------------------------ rule players
/// Max rolls (out of 36) with which the opponent can pin one of my blots in `s`
/// (mover's view, after the move).
pub fn max_blot_shots(s: &State) -> u32 {
    let f = s.flip();
    (1..25).filter(|&p| is_my_blot(s, p)).map(|p| pin_shots(&f, 25 - p)).max().unwrap_or(0)
}

/// Does move `m` (from `s`) obey the named rule of thumb?
pub fn obeys(rule: &str, s: &State, m: &Move) -> bool {
    let a = &m.after;
    if m.outcome > 0 {
        return true;
    }
    let new_pins = |lo: usize, hi: usize| (lo..=hi).any(|p| a.i_pin_at(p) && !s.i_pin_at(p));
    match rule {
        // never lift a home-board pin on purpose
        "nolift" => !(1..7).any(|p| s.i_pin_at(p) && !a.i_pin_at(p)),
        // trade-up exception allowed: lifting a 5/6 pin is fine if the move pins deeper (2-4)
        "nolift_tradeup" => !(1..7).any(|p| s.i_pin_at(p) && !a.i_pin_at(p) && !(p >= 5 && new_pins(2, 4))),
        // take a pin whenever one is available (anywhere)
        "alwayspin" => new_pins(1, 24),
        // take a pin in the home board whenever one is available
        "homepin" => new_pins(1, 6),
        // never leave a blot that 6+ rolls can pin
        "safe" => max_blot_shots(a) < 6,
        // never leave a single checker on the start (clear it in pairs)
        "pairs" => a.me[START] != 1,
        // before the bear-off, never add checkers to points 1-2
        "nobury" => {
            let outside: u32 = (7..25).map(|p| s.me[p] as u32).sum();
            outside == 0 || (a.me[1] + a.me[2]) <= (s.me[1] + s.me[2])
        }
        // while one of my checkers is pinned, don't bring more checkers into my home
        "nocrash" => {
            if s.opin == 0 { return true; }
            let home = |x: &State| (1..7).map(|p| x.me[p] as u32).sum::<u32>() + x.me[0] as u32;
            home(a) <= home(s)
        }
        // fairer versions: moves that make a new pin are always allowed
        "nobury2" => new_pins(1, 24) || obeys("nobury", s, m),
        "nocrash2" => new_pins(1, 24) || obeys("nocrash", s, m),
        // milder caution: never leave a blot that 12+ rolls (1 in 3) can pin
        "safe12" => max_blot_shots(a) < 12,
        // make as many points (2+ checkers) as possible: handled in RulePlayer (maximiser)
        "blocks" => true,
        _ => panic!("unknown rule {}", rule),
    }
}

fn made_points(s: &State) -> u32 {
    (1..25).filter(|&p| s.me[p] >= 2 || s.i_pin_at(p)).count() as u32
}

/// The net, restricted to moves that obey a rule (if any legal move does).
pub struct RulePlayer {
    rule: String,
    net: NetPlayer,
    pub decisions: u64,
    pub changed: u64,
}
impl RulePlayer {
    pub fn new(rule: &str, net: Arc<Net>) -> Self {
        RulePlayer { rule: rule.to_string(), net: NetPlayer::new(net), decisions: 0, changed: 0 }
    }
}
impl Player for RulePlayer {
    fn choose(&mut self, s: &State, moves: &[Move], rng: &mut Rng) -> usize {
        let free = self.net.choose(s, moves, rng);
        let allowed: Vec<usize> = if self.rule == "blocks" {
            let best = moves.iter().map(|m| made_points(&m.after)).max().unwrap_or(0);
            (0..moves.len()).filter(|&i| moves[i].outcome > 0 || made_points(&moves[i].after) == best).collect()
        } else {
            (0..moves.len()).filter(|&i| obeys(&self.rule, s, &moves[i])).collect()
        };
        self.decisions += 1;
        if allowed.is_empty() || allowed.contains(&free) {
            return free;
        }
        self.changed += 1;
        let mut best = (f32::NEG_INFINITY, allowed[0]);
        for &i in &allowed {
            let v = if moves[i].outcome > 0 { 10.0 * moves[i].outcome as f32 } else { self.net.move_value(&moves[i]) };
            if v > best.0 { best = (v, i); }
        }
        best.1
    }
}
impl Drop for RulePlayer {
    fn drop(&mut self) {
        eprintln!("RULESTATS {} decisions {} changed {}", self.rule, self.decisions, self.changed);
    }
}
