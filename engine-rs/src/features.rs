//! Shot counting and neural-network input encoding.

use crate::state::{State, START};

/// (d1, d2, weight out of 36)
pub const ROLLS: [(u8, u8, u8); 21] = {
    let mut r = [(0u8, 0u8, 0u8); 21];
    let mut i = 0;
    let mut a = 1;
    while a <= 6 {
        let mut b = a;
        while b <= 6 {
            r[i] = (a, b, if a == b { 1 } else { 2 });
            i += 1;
            b += 1;
        }
        a += 1;
    }
    r
};

#[inline]
fn can_land(s: &State, q: i32) -> bool {
    if q < 1 {
        return false;
    }
    let q = q as usize;
    let oc = s.op[25 - q];
    oc == 0 || (oc == 1 && (s.opin >> (25 - q)) & 1 == 0)
}

#[inline]
fn can_reach(s: &State, src: usize, target: usize, d1: u8, d2: u8) -> bool {
    let dist = src as i32 - target as i32;
    if dist <= 0 {
        return false;
    }
    let (d1, d2) = (d1 as i32, d2 as i32);
    let src = src as i32;
    if d1 == d2 {
        if dist % d1 != 0 || dist / d1 > 4 {
            return false;
        }
        for k in 1..=dist / d1 {
            if !can_land(s, src - k * d1) {
                return false;
            }
        }
        return true;
    }
    if dist == d1 || dist == d2 {
        return can_land(s, target as i32);
    }
    if dist == d1 + d2 {
        return can_land(s, target as i32) && (can_land(s, src - d1) || can_land(s, src - d2));
    }
    false
}

/// Rolls out of 36 with which the player to move can land on their point `target`.
pub fn pin_shots(s: &State, target: usize) -> u32 {
    let mut srcs = [0usize; 24];
    let mut n = 0;
    for p in target + 1..25 {
        if s.me[p] > 0 && !s.my_pinned_at(p) {
            srcs[n] = p;
            n += 1;
        }
    }
    if n == 0 {
        return 0;
    }
    let mut total = 0u32;
    for &(d1, d2, w) in ROLLS.iter() {
        if srcs[..n].iter().any(|&p| can_reach(s, p, target, d1, d2)) {
            total += w as u32;
        }
    }
    total
}

#[inline]
pub fn is_my_blot(s: &State, p: usize) -> bool {
    s.me[p] == 1 && !s.i_pin_at(p) && !s.my_pinned_at(p)
}

/// Inputs per side and in total.
pub const PER_SIDE: usize = 24 * 4 + 1 + 24 + 4;
pub const N_INPUTS: usize = 2 * PER_SIDE + 3;

fn encode_side(s: &State, opp_view: &State, out: &mut [f32]) {
    // s: this side to "own" (me), opp_view: s.flip() (used to count the other side's shots).
    let mut i = 0;
    for p in 1..25 {
        let c = s.me[p] as f32;
        out[i] = if c >= 1.0 { 1.0 } else { 0.0 };
        out[i + 1] = if c >= 2.0 { 1.0 } else { 0.0 };
        out[i + 2] = if c >= 3.0 { 1.0 } else { 0.0 };
        out[i + 3] = if c > 3.0 { (c - 3.0) / 12.0 } else { 0.0 };
        i += 4;
    }
    out[i] = s.me[0] as f32 / 15.0;
    i += 1;
    // Pin risk of each of my points: 1 if my checker there is pinned, else the
    // probability the other side can pin my blot there with their next roll.
    for p in 1..25 {
        out[i] = if s.my_pinned_at(p) {
            1.0
        } else if is_my_blot(s, p) {
            pin_shots(opp_view, 25 - p) as f32 / 36.0
        } else {
            0.0
        };
        i += 1;
    }
    // Mother exposed (exactly one free checker left on my start).
    out[i] = if s.me[START] == 1 && !s.my_pinned_at(START) { 1.0 } else { 0.0 };
    // Pins I hold in my home board: checkers stacked on them, and checkers below them
    // (those I can bear off before having to release).
    let mut stacked = 0u32;
    let mut deepest_home_pin = 0usize;
    for p in 1..7 {
        if s.i_pin_at(p) {
            stacked += s.me[p] as u32;
            deepest_home_pin = deepest_home_pin.max(p);
        }
    }
    let below: u32 = if deepest_home_pin > 0 {
        (1..deepest_home_pin).map(|p| s.me[p] as u32).sum::<u32>() + s.me[0] as u32
    } else {
        0
    };
    out[i + 1] = stacked as f32 / 15.0;
    out[i + 2] = below as f32 / 15.0;
    // Number of my pins anywhere.
    out[i + 3] = s.mpin.count_ones() as f32 / 5.0;
}

/// Encode a position (player to move = `s.me`).
pub fn encode(s: &State, out: &mut [f32]) {
    let f = s.flip();
    encode_side(s, &f, &mut out[..PER_SIDE]);
    encode_side(&f, s, &mut out[PER_SIDE..2 * PER_SIDE]);
    let (mp, op) = s.pips();
    let i = 2 * PER_SIDE;
    let diff = op as f32 - mp as f32; // > 0: I am ahead in the race
    out[i] = if diff > 0.0 { diff / 100.0 } else { 0.0 };
    out[i + 1] = if diff < 0.0 { -diff / 100.0 } else { 0.0 };
    // Race: no more contact possible (my highest checker below their lowest one).
    let my_high = (1..25).rev().find(|&p| s.me[p] > 0).unwrap_or(0);
    let their_low = (1..25).find(|&p| s.opp_at(p) > 0).unwrap_or(25);
    out[i + 2] = if my_high < their_low { 1.0 } else { 0.0 };
}
