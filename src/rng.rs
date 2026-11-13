//! The random numbers used to pick a row.

use std::cell::Cell;
use std::hash::{BuildHasher, RandomState};

thread_local! {
    /// Seeded per thread, by the random keys of the standard library its hasher.
    static STATE: Cell<u64> = Cell::new(RandomState::new().hash_one(0x5EED) | 1);
}

/// A random number below the given limit, or zero if the limit is zero.
///
/// Uniform enough to pick a row, not fit for cryptography.
#[must_use]
pub(crate) fn below(limit: usize) -> usize {
    let random = STATE.with(|state| {
        // xorshift64*
        let mut x = state.get();
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        state.set(x);
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    });
    ((u128::from(random) * limit as u128) >> 64) as usize
}

#[cfg(test)]
mod tests {
    use super::below;

    #[test]
    fn stays_below_limit() {
        assert_eq!(below(0), 0);
        assert_eq!(below(1), 0);
        for limit in [2, 3, 10, 1000, usize::MAX] {
            assert!((0..1000).all(|_| below(limit) < limit));
        }
        // large limits are not cut short
        assert!((0..100).any(|_| below(usize::MAX) > usize::MAX / 2));
    }

    #[test]
    fn reaches_every_value() {
        let mut seen = [0usize; 8];
        for _ in 0..8000 {
            seen[below(8)] += 1;
        }
        assert!(seen.iter().all(|n| (700..1300).contains(n)), "{seen:?}");
    }
}
