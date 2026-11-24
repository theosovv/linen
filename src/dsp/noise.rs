#[derive(Clone, Debug, PartialEq)]
pub struct Noise {
    initial_seed: u64,
    state: u64,
}

impl Noise {
    pub const fn new(seed: u64) -> Self {
        Self {
            initial_seed: seed,
            state: seed,
        }
    }

    pub const fn initial_seed(&self) -> u64 {
        self.initial_seed
    }

    pub const fn state(&self) -> u64 {
        self.state
    }

    pub fn reset(&mut self) {
        self.state = self.initial_seed;
    }

    pub fn next_sample(&mut self) -> f32 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);

        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;

        (z >> 40) as f32 / 8_388_608.0 - 1.0
    }
}
