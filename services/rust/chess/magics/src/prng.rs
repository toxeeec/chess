// https://github.com/official-stockfish/Stockfish/blob/master/src/misc.h

pub struct Prng {
    seed: u64,
}

impl Prng {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    fn rand64(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        self.seed.wrapping_mul(2685821657736338717)
    }

    pub fn sparse_rand<T: From<u64>>(&mut self) -> T {
        T::from(self.rand64() & self.rand64() & self.rand64())
    }
}
