//! Look-ahead: 2-ply and 3-ply evaluation and move choice.
//!
//! Naming follows the rest of the project: "1-ply" = pick the move whose resulting position the
//! net rates best. "2-ply" = for each candidate move, average over the opponent's 21 rolls the
//! result of the opponent's best (1-ply) reply. In gnubg terms these are 0-ply and 1-ply.
//!
//! A move filter keeps 2-ply affordable: candidates are ranked at 1-ply and only the best `keep`
//! moves within `margin` points of the best one are searched. Moves far below the best at 1-ply
//! are almost never best at 2-ply, so pruning them costs little accuracy.
//!
//! 3-ply adds one more roll: for each candidate move, the opponent's 21 rolls, each answered
//! with the opponent's best reply chosen at 2-ply among its filtered candidates. It is about
//! 30-60 times the cost of 2-ply, so it is used for checking individual positions.

use crate::features::ROLLS;
use crate::nn::{equity, invert, terminal, Net, Scratch};
use crate::state::{Move, MoveGen, State};
use std::sync::Arc;

pub struct Searcher {
    pub net: Arc<Net>,
    sc: Scratch,
    gen: MoveGen,
    buf: Vec<Move>,
    pub keep: usize,
    pub margin: f32,
}

impl Searcher {
    pub fn new(net: Arc<Net>, keep: usize, margin: f32) -> Self {
        let sc = Scratch::new(&net);
        Searcher { net, sc, gen: MoveGen::default(), buf: Vec::with_capacity(64), keep, margin }
    }

    /// 1-ply outputs [W, WD, LD] of a move, from the mover's point of view.
    #[inline]
    pub fn out1(&mut self, m: &Move) -> [f32; 3] {
        if m.outcome > 0 { terminal(m.outcome) } else { invert(self.net.eval(&m.after.flip(), &mut self.sc)) }
    }

    /// 2-ply outputs of a position for the player on roll: for each of the 21 rolls, the player's
    /// best move by 1-ply value; the outputs of those moves averaged over the rolls (weights /36).
    pub fn eval2(&mut self, s: &State) -> [f32; 3] {
        let mut acc = [0f32; 3];
        for &(d1, d2, w) in ROLLS.iter() {
            let mut moves = std::mem::take(&mut self.buf);
            self.gen.legal_moves(s, d1, d2, &mut moves);
            let o = if moves.is_empty() {
                invert(self.net.eval(&s.flip(), &mut self.sc)) // no legal move: the turn passes
            } else {
                let mut best = (f32::NEG_INFINITY, [0f32; 3]);
                for m in moves.iter() {
                    let o = self.out1(m);
                    let v = if m.outcome > 0 { 10.0 * m.outcome as f32 } else { equity(o) };
                    if v > best.0 {
                        best = (v, o);
                    }
                }
                best.1
            };
            self.buf = moves;
            for k in 0..3 {
                acc[k] += w as f32 * o[k];
            }
        }
        acc.map(|x| x / 36.0)
    }

    /// 2-ply outputs of a move, from the mover's point of view.
    pub fn out2(&mut self, m: &Move) -> [f32; 3] {
        if m.outcome > 0 { terminal(m.outcome) } else { invert(self.eval2(&m.after.flip())) }
    }

    /// Pick the best of `moves` for the player to move by the value at `depth` (1 or 2), searching
    /// only the moves that survive the 1-ply filter with (keep, margin). Returns its outputs.
    fn best_outputs(&mut self, moves: &[Move], depth: u8, keep: usize, margin: f32) -> [f32; 3] {
        if let Some(m) = moves.iter().filter(|m| m.outcome > 0).max_by_key(|m| m.outcome) {
            return terminal(m.outcome);
        }
        let (k0, m0) = (self.keep, self.margin);
        self.keep = keep;
        self.margin = margin;
        let cands = self.filter(moves);
        self.keep = k0;
        self.margin = m0;
        if depth == 1 || cands.len() == 1 {
            return self.out1(&moves[cands[0].0]);
        }
        let mut best = (f32::NEG_INFINITY, [0f32; 3]);
        for (i, _) in cands {
            let o = self.out2(&moves[i]);
            let e = equity(o);
            if e > best.0 {
                best = (e, o);
            }
        }
        best.1
    }

    /// 3-ply outputs for the player on roll: each of the 21 rolls answered with the best move
    /// chosen at 2-ply among the 1-ply top `keep3` within `margin3`.
    pub fn eval3(&mut self, s: &State, keep3: usize, margin3: f32) -> [f32; 3] {
        let mut acc = [0f32; 3];
        let mut moves = Vec::with_capacity(64);
        for &(d1, d2, w) in ROLLS.iter() {
            self.gen.legal_moves(s, d1, d2, &mut moves);
            let o = if moves.is_empty() {
                invert(self.eval2(&s.flip())) // no legal move: the turn passes
            } else {
                let mv = moves.clone();
                self.best_outputs(&mv, 2, keep3, margin3)
            };
            for k in 0..3 {
                acc[k] += w as f32 * o[k];
            }
        }
        acc.map(|x| x / 36.0)
    }

    /// 3-ply outputs of a move, from the mover's point of view.
    pub fn out3(&mut self, m: &Move, keep3: usize, margin3: f32) -> [f32; 3] {
        if m.outcome > 0 { terminal(m.outcome) } else { invert(self.eval3(&m.after.flip(), keep3, margin3)) }
    }

    /// Indices of the moves that survive the filter, best 1-ply first, with their 1-ply equity.
    pub fn filter(&mut self, moves: &[Move]) -> Vec<(usize, f32)> {
        let mut v: Vec<(usize, f32)> = (0..moves.len())
            .map(|i| {
                let m = &moves[i];
                (i, if m.outcome > 0 { 10.0 * m.outcome as f32 } else { equity(self.out1(m)) })
            })
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let best = v[0].1;
        let keep = self.keep.max(1);
        v.into_iter().enumerate().filter(|(r, (_, e))| *r < keep && best - e <= self.margin).map(|(_, x)| x).collect()
    }

    /// Choose a move by 2-ply value among the filtered candidates. Returns (index, 2-ply equity).
    pub fn choose2(&mut self, moves: &[Move]) -> (usize, f32) {
        if let Some(i) = (0..moves.len()).filter(|&i| moves[i].outcome > 0).max_by_key(|&i| moves[i].outcome) {
            return (i, moves[i].outcome as f32);
        }
        let cands = self.filter(moves);
        if cands.len() == 1 {
            return cands[0]; // nothing to compare: skip the expensive search (1-ply equity returned)
        }
        let mut best = (f32::NEG_INFINITY, cands[0].0);
        for (i, _) in cands {
            let e = equity(self.out2(&moves[i]));
            if e > best.0 {
                best = (e, i);
            }
        }
        (best.1, best.0)
    }

    /// Choose a move at 3-ply: 1-ply filter, then 2-ply for the survivors, then 3-ply for the
    /// best `keep3` of those within `margin3` (2-ply equity). Returns (index, equity at the
    /// deepest level that was needed).
    pub fn choose3(&mut self, moves: &[Move], keep3: usize, margin3: f32) -> (usize, f32) {
        if let Some(i) = (0..moves.len()).filter(|&i| moves[i].outcome > 0).max_by_key(|&i| moves[i].outcome) {
            return (i, moves[i].outcome as f32);
        }
        let cands = self.filter(moves);
        if cands.len() == 1 {
            return cands[0];
        }
        let mut two: Vec<(usize, f32)> = cands.iter().map(|&(i, _)| (i, equity(self.out2(&moves[i])))).collect();
        two.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top = two[0].1;
        let finalists: Vec<usize> = two.iter().enumerate().filter(|(r, (_, e))| *r < keep3 && top - e <= margin3).map(|(_, &(i, _))| i).collect();
        if finalists.len() == 1 {
            return two[0];
        }
        let mut best = (f32::NEG_INFINITY, finalists[0]);
        for i in finalists {
            let e = equity(self.out3(&moves[i], 3, 0.12));
            if e > best.0 {
                best = (e, i);
            }
        }
        (best.1, best.0)
    }
}
