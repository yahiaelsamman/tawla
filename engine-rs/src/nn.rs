//! One-hidden-layer perceptron with three sigmoid outputs: W (win), WD (win double),
//! LD (lose double), from the point of view of the player to move.

use crate::features::{encode, N_INPUTS};
use crate::rng::Rng;
use crate::state::State;
use std::io::{Read, Write};

#[derive(Clone)]
pub struct Net {
    pub n_in: usize,
    pub n_hid: usize,
    pub w1: Vec<f32>, // n_hid x n_in
    pub b1: Vec<f32>,
    pub w2: Vec<f32>, // 3 x n_hid
    pub b2: Vec<f32>,
    pub games_trained: u64,
}

#[inline]
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

pub struct Scratch {
    pub x: Vec<f32>,
    pub h: Vec<f32>,
}

impl Scratch {
    pub fn new(net: &Net) -> Self {
        Scratch { x: vec![0.0; net.n_in], h: vec![0.0; net.n_hid] }
    }
}

impl Net {
    pub fn new(n_hid: usize, rng: &mut Rng) -> Self {
        let n_in = N_INPUTS;
        let mut r = |scale: f32| (rng.f32() * 2.0 - 1.0) * scale;
        let w1 = (0..n_hid * n_in).map(|_| r(0.1)).collect();
        let b1 = (0..n_hid).map(|_| r(0.1)).collect();
        let w2 = (0..3 * n_hid).map(|_| r(0.1)).collect();
        let b2 = vec![0.0; 3];
        Net { n_in, n_hid, w1, b1, w2, b2, games_trained: 0 }
    }

    /// Forward pass on already-encoded inputs in sc.x; fills sc.h, returns outputs.
    #[inline]
    pub fn forward(&self, sc: &mut Scratch) -> [f32; 3] {
        let n_in = self.n_in;
        for j in 0..self.n_hid {
            let row = &self.w1[j * n_in..(j + 1) * n_in];
            let mut acc = self.b1[j];
            for (w, x) in row.iter().zip(sc.x.iter()) {
                acc += w * x;
            }
            sc.h[j] = sigmoid(acc);
        }
        let mut out = [0f32; 3];
        for k in 0..3 {
            let row = &self.w2[k * self.n_hid..(k + 1) * self.n_hid];
            let mut acc = self.b2[k];
            for (w, h) in row.iter().zip(sc.h.iter()) {
                acc += w * h;
            }
            out[k] = sigmoid(acc);
        }
        out
    }

    pub fn eval(&self, s: &State, sc: &mut Scratch) -> [f32; 3] {
        encode(s, &mut sc.x);
        self.forward(sc)
    }

    /// One gradient step moving the outputs for the inputs in sc.x toward `target`
    /// (squared error). `sc` must hold the forward pass for these inputs.
    pub fn train(&mut self, sc: &mut Scratch, out: [f32; 3], target: [f32; 3], alpha: f32) {
        let n_in = self.n_in;
        let n_hid = self.n_hid;
        let mut delta_o = [0f32; 3];
        for k in 0..3 {
            delta_o[k] = (target[k] - out[k]) * out[k] * (1.0 - out[k]);
        }
        for j in 0..n_hid {
            let h = sc.h[j];
            let mut back = 0f32;
            for k in 0..3 {
                back += delta_o[k] * self.w2[k * n_hid + j];
            }
            let delta_h = back * h * (1.0 - h);
            for k in 0..3 {
                self.w2[k * n_hid + j] += alpha * delta_o[k] * h;
            }
            if delta_h != 0.0 {
                let row = &mut self.w1[j * n_in..(j + 1) * n_in];
                let a = alpha * delta_h;
                for (w, x) in row.iter_mut().zip(sc.x.iter()) {
                    if *x != 0.0 {
                        *w += a * x;
                    }
                }
                self.b1[j] += a;
            }
        }
        for k in 0..3 {
            self.b2[k] += alpha * delta_o[k];
        }
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut f = std::fs::File::create(path)?;
        f.write_all(b"MHBN")?;
        for v in [self.n_in as u64, self.n_hid as u64, self.games_trained] {
            f.write_all(&v.to_le_bytes())?;
        }
        for vec in [&self.w1, &self.b1, &self.w2, &self.b2] {
            for x in vec.iter() {
                f.write_all(&x.to_le_bytes())?;
            }
        }
        Ok(())
    }

    pub fn load(path: &str) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        std::fs::File::open(path)?.read_to_end(&mut buf)?;
        assert_eq!(&buf[..4], b"MHBN", "not a Mahbusa net file");
        let u = |i: usize| u64::from_le_bytes(buf[4 + 8 * i..12 + 8 * i].try_into().unwrap());
        let (n_in, n_hid, games) = (u(0) as usize, u(1) as usize, u(2));
        assert_eq!(n_in, N_INPUTS, "input size mismatch: net {} vs code {}", n_in, N_INPUTS);
        let mut pos = 28;
        let mut take = |n: usize| -> Vec<f32> {
            let v = (0..n)
                .map(|i| f32::from_le_bytes(buf[pos + 4 * i..pos + 4 * i + 4].try_into().unwrap()))
                .collect();
            pos += 4 * n;
            v
        };
        let w1 = take(n_hid * n_in);
        let b1 = take(n_hid);
        let w2 = take(3 * n_hid);
        let b2 = take(3);
        Ok(Net { n_in, n_hid, w1, b1, w2, b2, games_trained: games })
    }
}

/// Expected points for the player to move: 2W - 1 + WD - LD.
#[inline]
pub fn equity(o: [f32; 3]) -> f32 {
    2.0 * o[0] - 1.0 + o[1] - o[2]
}

/// Outputs seen from the other player: W' = 1 - W, WD' = LD, LD' = WD.
#[inline]
pub fn invert(o: [f32; 3]) -> [f32; 3] {
    [1.0 - o[0], o[2], o[1]]
}

/// Outputs (mover's view) for a finished game won with `points`.
#[inline]
pub fn terminal(points: i8) -> [f32; 3] {
    [1.0, if points >= 2 { 1.0 } else { 0.0 }, 0.0]
}
