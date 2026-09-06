//! Seeded RNG with named sub-streams so adding random calls in one
//! system never reshuffles another system's rolls.

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimRng {
    seed: u64,
    streams: BTreeMap<String, ChaCha20Rng>,
}

impl SimRng {
    pub fn new(seed: u64) -> Self {
        Self { seed, streams: BTreeMap::new() }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Get (or lazily create) a named stream. Stream seed = hash(master seed, name).
    pub fn stream(&mut self, name: &str) -> &mut ChaCha20Rng {
        let seed = self.seed;
        self.streams.entry(name.to_string()).or_insert_with(|| {
            let mut key = [0u8; 32];
            key[..8].copy_from_slice(&seed.to_le_bytes());
            for (i, b) in name.bytes().enumerate().take(24) {
                key[8 + i] = b;
            }
            ChaCha20Rng::from_seed(key)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn streams_are_independent_and_deterministic() {
        let mut a = SimRng::new(42);
        let mut b = SimRng::new(42);
        let x: u32 = a.stream("events").gen();
        let _: u32 = a.stream("weather").gen();
        let y: u32 = b.stream("events").gen();
        assert_eq!(x, y);
    }
}
