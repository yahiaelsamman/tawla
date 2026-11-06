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
        format!(
            "{} games  {} vs {}: win {:.1}% ±{:.1}  ppg {:+.3}  doubles {:.1}%/{:.1}%  mother-pin {:.1}%  avg turns {:.1}",
            self.games,
            a,
            b,
            100.0 * w,
            196.0 * (w * (1.0 - w) / n).sqrt(),
            self.ppg(),
            100.0 * self.doubles[0] as f64 / n,
            100.0 * self.doubles[1] as f64 / n,
            100.0 * self.mothers as f64 / n,
            self.turns as f64 / n
        )
    }
}

pub fn make_player(spec: &str) -> Box<dyn Player + Send> {
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
