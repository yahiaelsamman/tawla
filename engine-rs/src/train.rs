//! TD(0) self-play training, Hogwild-style across threads (shared weights, no locks).

use crate::nn::{equity, invert, terminal, Net, Scratch};
use crate::rng::Rng;
use crate::state::{MoveGen, State};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

pub struct Shared(UnsafeCell<Net>);
unsafe impl Sync for Shared {}
unsafe impl Send for Shared {}

impl Shared {
    pub fn new(net: Net) -> Self {
        Shared(UnsafeCell::new(net))
    }
    #[allow(clippy::mut_from_ref)]
    pub fn get(&self) -> &mut Net {
        unsafe { &mut *self.0.get() }
    }
}

/// Play one self-play game and update the net after every move.
pub fn train_game(net: &mut Net, alpha: f32, rng: &mut Rng, gen: &mut MoveGen, sa: &mut Scratch, sb: &mut Scratch) -> u32 {
    let mut s = State::initial();
    let mut moves = Vec::with_capacity(64);
    for turn in 1..5000u32 {
        let (d1, d2) = (rng.die(), rng.die());
        gen.legal_moves(&s, d1, d2, &mut moves);
        let out_s = net.eval(&s, sa);
        if moves.is_empty() {
            let f = s.flip();
            let tgt = invert(net.eval(&f, sb));
            net.train(sa, out_s, tgt, alpha);
            s = f;
            continue;
        }
        let mut best_v = f32::NEG_INFINITY;
        let mut best_i = 0;
        let mut best_t = [0f32; 3];
        for (i, m) in moves.iter().enumerate() {
            let (v, t) = if m.outcome > 0 {
                (10.0 * m.outcome as f32, terminal(m.outcome))
            } else {
                let o = net.eval(&m.after.flip(), sb);
                (-equity(o), invert(o))
            };
            if v > best_v {
                best_v = v;
                best_i = i;
                best_t = t;
            }
        }
        // sa still holds the forward pass of s (sb was used for the candidates).
        net.train(sa, out_s, best_t, alpha);
        let m = moves[best_i];
        if m.outcome > 0 {
            return turn;
        }
        s = m.after.flip();
    }
    panic!("training game did not finish");
}

pub struct TrainCfg {
    pub games: u64,
    pub threads: usize,
    pub alpha: f32,
    pub seed: u64,
}

/// Train `net` for cfg.games games. `on_progress(net_snapshot, games_done)` is called
/// from the main thread roughly every `report_every` games.
pub fn train(net: Net, cfg: &TrainCfg, report_every: u64, mut on_progress: impl FnMut(&Net, u64)) -> Net {
    let base_games = net.games_trained;
    let shared = Arc::new(Shared::new(net));
    let done = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let target = cfg.games;
    let handles: Vec<_> = (0..cfg.threads)
        .map(|t| {
            let shared = shared.clone();
            let done = done.clone();
            let stop = stop.clone();
            let alpha = cfg.alpha;
            let seed = cfg.seed.wrapping_mul(7919).wrapping_add(t as u64 * 104729 + base_games);
            std::thread::spawn(move || {
                let mut rng = Rng::new(seed);
                let mut gen = MoveGen::default();
                let net = shared.get();
                let mut sa = Scratch::new(net);
                let mut sb = Scratch::new(net);
                while !stop.load(Ordering::Relaxed) {
                    let n = done.fetch_add(1, Ordering::Relaxed);
                    if n >= target {
                        break;
                    }
                    train_game(shared.get(), alpha, &mut rng, &mut gen, &mut sa, &mut sb);
                }
            })
        })
        .collect();
    let mut next_report = report_every;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let n = done.load(Ordering::Relaxed).min(target);
        if n >= next_report || n >= target {
            let mut snap = shared.get().clone();
            snap.games_trained = base_games + n;
            on_progress(&snap, n);
            next_report = (n / report_every + 1) * report_every;
        }
        if n >= target {
            break;
        }
    }
    stop.store(true, Ordering::Relaxed);
    for h in handles {
        h.join().unwrap();
    }
    let mut net = shared.get().clone();
    net.games_trained = base_games + target;
    net
}
