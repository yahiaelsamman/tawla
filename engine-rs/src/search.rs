//! Look-ahead: 2-ply evaluation and move choice.
//!
//! Naming follows the rest of the project: "1-ply" = pick the move whose resulting position the
//! net rates best. "2-ply" = for each candidate move, average over the opponent's 21 rolls the
//! result of the opponent's best (1-ply) reply. In gnubg terms these are 0-ply and 1-ply.
//!
//! A move filter keeps 2-ply affordable: candidates are ranked at 1-ply and only the best `keep`
//! moves within `margin` points of the best one are searched. Moves far below the best at 1-ply
//! are almost never best at 2-ply, so pruning them costs little accuracy.

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
}
