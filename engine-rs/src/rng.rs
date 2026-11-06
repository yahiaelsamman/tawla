//! Small, fast PRNG (xoshiro256**), seeded with splitmix64.

pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut s = [0u64; 4];
        for x in s.iter_mut() {
            z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut v = z;
            v = (v ^ (v >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            v = (v ^ (v >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            *x = v ^ (v >> 31);
        }
        Rng { s }
    }
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }
    #[inline]
    pub fn below(&mut self, n: u64) -> u64 {
        ((self.next_u64() >> 32) * n) >> 32
    }
    #[inline]
    pub fn die(&mut self) -> u8 {
        self.below(6) as u8 + 1
    }
    #[inline]
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}
