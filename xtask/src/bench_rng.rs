//! Deterministic pseudo-random numbers and distinct position picks for bench data.

use std::collections::HashSet;

/// A xorshift64* generator whose state starts as `splitmix64(seed)`.
pub struct Rng {
    /// Current xorshift state.
    state: u64,
}

impl Rng {
    /// Builds a generator for `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            state: splitmix64(seed),
        }
    }

    /// Returns the next 64 pseudo-random bits.
    pub fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Fills `buffer` with pseudo-random bytes.
    pub fn fill(&mut self, buffer: &mut [u8]) {
        for chunk in buffer.chunks_mut(8) {
            let value = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&value[..chunk.len()]);
        }
    }
}

/// Mixes one seed into the xorshift state.
pub fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = value;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// Picks `count` distinct positions in `0..range`, sorted, skipping duplicates.
pub fn pick_positions(rng: &mut Rng, range: u64, count: u64) -> Vec<u64> {
    let wanted = count.min(range) as usize;
    if wanted == 0 {
        return Vec::new();
    }
    let mut positions = Vec::with_capacity(wanted);
    let mut seen = HashSet::with_capacity(wanted);
    while positions.len() < wanted {
        let candidate = rng.next_u64() % range;
        if seen.insert(candidate) {
            positions.push(candidate);
        }
    }
    positions.sort_unstable();
    positions
}
