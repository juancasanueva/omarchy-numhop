//! A tiny seeded random number generator.
//!
//! The domain owns its randomness so question generation is deterministic under test
//! and the game does not depend on an external crate whose API changes between releases.

#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // A zero state would make xorshift return zeros forever, so mix the seed first.
        let mut rng = Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        };
        if rng.state == 0 {
            rng.state = 0xDEAD_BEEF;
        }
        rng.next_u64();
        rng
    }

    /// Seeds from the wall clock, for real games.
    pub fn from_time() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        Self::new(nanos)
    }

    pub fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A uniform integer in `lo..=hi`. Returns `lo` when the range is empty.
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u64;
        lo + (self.next_u64() % span) as i64
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        let unit = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        unit < p
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.range(0, items.len() as i64 - 1) as usize]
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.range(0, i as i64) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_stays_inside_bounds() {
        let mut rng = Rng::new(7);
        for _ in 0..10_000 {
            let v = rng.range(-3, 5);
            assert!((-3..=5).contains(&v));
        }
    }

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(123);
        let mut b = Rng::new(123);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn empty_range_returns_low() {
        let mut rng = Rng::new(1);
        assert_eq!(rng.range(4, 4), 4);
        assert_eq!(rng.range(9, 2), 9);
    }
}
