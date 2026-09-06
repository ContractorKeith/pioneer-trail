//! Portable world codes include the seed, trail and era, with a typo checksum.

use anyhow::{bail, ensure, Result};

const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSeed {
    pub seed: u64,
    pub trail: String,
    pub era: String,
}

impl WorldSeed {
    pub fn code(&self) -> Result<String> {
        let trail = match self.trail.as_str() {
            "oregon" => 0,
            "california" => 1,
            "mormon" => 2,
            _ => bail!("Unknown trail in seed code"),
        };
        let era = match self.era.as_str() {
            "1843" => 0,
            "1848" => 1,
            "1852" => 2,
            "1866" => 3,
            _ => bail!("Unknown era in seed code"),
        };
        let payload = (u128::from(self.seed) << 4) | (trail << 2) | era;
        let check = checksum(payload);
        let mut value = (payload << 5) | u128::from(check);
        let mut digits = [b'0'; 15];
        for digit in digits.iter_mut().rev() {
            *digit = ALPHABET[(value & 31) as usize];
            value >>= 5;
        }
        Ok(format!("pt-{}", std::str::from_utf8(&digits)?))
    }

    pub fn parse(code: &str) -> Result<Self> {
        let code = code.trim().to_ascii_lowercase();
        let digits =
            code.strip_prefix("pt-").ok_or_else(|| anyhow::anyhow!("Seed codes start with pt-"))?;
        ensure!(digits.len() == 15, "Seed codes contain exactly 15 letters and digits after pt-");
        let mut value = 0u128;
        for digit in digits.bytes() {
            let n = ALPHABET
                .iter()
                .position(|&candidate| candidate == digit)
                .ok_or_else(|| anyhow::anyhow!("Invalid letter or digit in seed code"))?;
            value = (value << 5) | n as u128;
        }
        let payload = value >> 5;
        ensure!(payload >> 68 == 0, "Seed code is outside the supported range");
        ensure!(
            checksum(payload) == (value & 31) as u8,
            "Seed checksum does not match; check the code"
        );
        let trail = match (payload >> 2) & 3 {
            0 => "oregon",
            1 => "california",
            2 => "mormon",
            _ => bail!("Unknown trail in seed code"),
        };
        let era = ["1843", "1848", "1852", "1866"][(payload & 3) as usize];
        Ok(Self { seed: (payload >> 4) as u64, trail: trail.into(), era: era.into() })
    }
}

fn checksum(mut payload: u128) -> u8 {
    let mut result = 19u8;
    for _ in 0..14 {
        result = ((result << 1) | (result >> 4)) & 31;
        result ^= (payload & 31) as u8;
        payload >>= 5;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip_every_trail_era_and_seed_boundary() {
        for seed in [0, 1, 42, u64::MAX] {
            for trail in ["oregon", "california", "mormon"] {
                for era in ["1843", "1848", "1852", "1866"] {
                    let world = WorldSeed { seed, trail: trail.into(), era: era.into() };
                    assert_eq!(WorldSeed::parse(&world.code().unwrap()).unwrap(), world);
                }
            }
        }
    }

    #[test]
    fn single_character_changes_are_rejected() {
        let world = WorldSeed { seed: 42, trail: "oregon".into(), era: "1848".into() };
        let code = world.code().unwrap();
        for index in 3..code.len() {
            for &digit in ALPHABET {
                let mut changed = code.as_bytes().to_vec();
                if changed[index] == digit {
                    continue;
                }
                changed[index] = digit;
                assert!(WorldSeed::parse(std::str::from_utf8(&changed).unwrap()).is_err());
            }
        }
        assert!(WorldSeed::parse("pt-zzzzzzzzzzzzzzz").is_err());
        assert!(WorldSeed::parse("42").is_err());
    }
}
