//! A column of which the rows can be changed while the database is in use.

use crate::rowset::{RowSet, WORD};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

/// The number of words in the first segment, each next one is twice the size of the one before.
const FIRST: usize = 64;

/// The number of segments it takes to hold [`RowSet::MAX_ROWS`] rows.
const SEGMENTS: usize = 21;

/// A set of rows that can be changed at any time, from any thread, without a lock.
///
/// Use it for state that changes while the database is queried,
/// such as rows that are disabled for a while. The column is not part
/// of the database and knows nothing of its size: any row can be set,
/// also the rows appended to the database after the column was created.
///
/// Memory is taken for the rows up to the last one set, rounded up to
/// at most twice as many. Cloning is cheap, clones share the same rows.
#[derive(Debug, Clone, Default)]
pub struct RuntimeColumn {
    /// Segments are created when a row in them is first set, and never move after:
    /// that is what allows the column to grow while it is read.
    segments: Arc<[OnceLock<Box<[AtomicU64]>>; SEGMENTS]>,
}

/// The segment of a word, and the place of the word in it.
const fn place(word: usize) -> (usize, usize) {
    let segment = (word / FIRST + 1).ilog2() as usize;
    (segment, word - FIRST * ((1 << segment) - 1))
}

/// The number of words in a segment.
const fn words_in(segment: usize) -> usize {
    FIRST << segment
}

impl RuntimeColumn {
    /// Create a column, with none of its rows set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return `true` if the row is set.
    #[must_use]
    pub fn get(&self, row: usize) -> bool {
        let (segment, at) = place(row / WORD);
        self.segments
            .get(segment)
            .and_then(OnceLock::get)
            .is_some_and(|words| words[at].load(Ordering::Relaxed) >> (row % WORD) & 1 == 1)
    }

    /// Set or unset a row, returns the value it had before.
    ///
    /// # Panics
    ///
    /// Panics if the row is not below [`RowSet::MAX_ROWS`].
    #[expect(
        clippy::must_use_candidate,
        reason = "the previous value is seldom of use"
    )]
    pub fn set(&self, row: usize, value: bool) -> bool {
        assert!(
            row < RowSet::MAX_ROWS,
            "venndb: a database holds at most u32::MAX rows"
        );
        let (segment, at) = place(row / WORD);
        let bit = 1 << (row % WORD);
        let before = if value {
            let words = self.segments[segment]
                .get_or_init(|| (0..words_in(segment)).map(|_| AtomicU64::new(0)).collect());
            words[at].fetch_or(bit, Ordering::Relaxed)
        } else {
            // no row of a segment that was never created is set
            match self.segments[segment].get() {
                Some(words) => words[at].fetch_and(!bit, Ordering::Relaxed),
                None => 0,
            }
        };
        before & bit != 0
    }

    /// Set a row, returns `true` only for the one caller that did set it.
    ///
    /// # Panics
    ///
    /// Panics if the row is not below [`RowSet::MAX_ROWS`].
    #[must_use]
    pub fn claim(&self, row: usize) -> bool {
        !self.set(row, true)
    }

    /// Unset all rows.
    pub fn clear(&self) {
        for word in self.segments.iter().filter_map(OnceLock::get).flatten() {
            word.store(0, Ordering::Relaxed);
        }
    }

    /// The number of rows set.
    #[must_use]
    pub fn count(&self) -> usize {
        self.segments
            .iter()
            .filter_map(OnceLock::get)
            .flatten()
            .map(|word| word.load(Ordering::Relaxed).count_ones() as usize)
            .sum()
    }

    /// Iterate over the rows set, in increasing order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.segments
            .iter()
            .enumerate()
            .filter_map(|(segment, words)| Some((segment, words.get()?)))
            .flat_map(|(segment, words)| {
                let first = FIRST * ((1 << segment) - 1);
                words.iter().enumerate().flat_map(move |(at, word)| {
                    let word = word.load(Ordering::Relaxed);
                    (0..WORD)
                        .filter(move |bit| word >> bit & 1 == 1)
                        .map(move |bit| (first + at) * WORD + bit)
                })
            })
    }

    /// Keep the bits of the mask of which the row is set, or is not set, in the column.
    pub(crate) fn filter(&self, mask: &mut [u64], value: bool) {
        let mut rest = mask;
        for (segment, words) in self.segments.iter().enumerate() {
            if rest.is_empty() {
                break;
            }
            let (part, later) = rest.split_at_mut(words_in(segment).min(rest.len()));
            match (words.get(), value) {
                (Some(words), true) => {
                    for (word, other) in part.iter_mut().zip(words.iter()) {
                        *word &= other.load(Ordering::Relaxed);
                    }
                }
                (Some(words), false) => {
                    for (word, other) in part.iter_mut().zip(words.iter()) {
                        *word &= !other.load(Ordering::Relaxed);
                    }
                }
                (None, true) => part.fill(0),
                (None, false) => (),
            }
            rest = later;
        }
        // beyond the last segment there are no rows to be set
        if value {
            rest.fill(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_hold_every_row() {
        assert_eq!(place(0), (0, 0));
        assert_eq!(place(FIRST - 1), (0, FIRST - 1));
        assert_eq!(place(FIRST), (1, 0));
        assert_eq!(place(FIRST * 3 - 1), (1, FIRST * 2 - 1));
        assert_eq!(place(FIRST * 3), (2, 0));
        assert_eq!(
            (words_in(0), words_in(1), words_in(2)),
            (FIRST, FIRST * 2, FIRST * 4)
        );
        // each word has one place, in the segment after that of the word before it or the same
        let mut last = (0, 0);
        for word in 1..FIRST * 40 {
            let (segment, at) = place(word);
            assert!(at < words_in(segment));
            assert!((segment, at) == (last.0, last.1 + 1) || (segment, at) == (last.0 + 1, 0));
            last = (segment, at);
        }
        let (segment, at) = place((RowSet::MAX_ROWS - 1) / WORD);
        assert!(segment < SEGMENTS && at < words_in(segment));
        assert_eq!(segment, SEGMENTS - 1);
    }

    #[test]
    fn set_and_get() {
        let column = RuntimeColumn::new();
        assert_eq!(column.count(), 0);
        assert!(!column.get(0) && !column.get(usize::MAX));
        assert!(!column.set(0, true));
        assert!(!column.set(129, true));
        assert!(!column.set(64, true));
        assert!(column.set(64, true));
        assert!(column.get(0) && column.get(64) && column.get(129));
        assert!(!column.get(1) && !column.get(130));
        assert_eq!(column.count(), 3);
        assert_eq!(column.iter().collect::<Vec<_>>(), [0, 64, 129]);
        assert!(column.set(64, false));
        assert!(!column.set(64, false));
        assert!(!column.get(64) && column.get(0));
        assert_eq!(column.count(), 2);
        column.clear();
        assert_eq!(column.count(), 0);
        assert_eq!(column.iter().next(), None);
    }

    #[test]
    fn grows_with_the_rows_set() {
        let column = RuntimeColumn::new();
        let created = |column: &RuntimeColumn| {
            let segments = column.segments.iter();
            segments.map(|s| s.get().is_some()).collect::<Vec<_>>()
        };
        assert!(created(&column).iter().all(|created| !created));
        // unsetting a row creates nothing
        assert!(!column.set(1_000_000, false));
        assert!(created(&column).iter().all(|created| !created));

        // the last segments hold billions of rows: too much to create in a test
        let rows = [5, FIRST * WORD - 1, FIRST * WORD, 1_000_000];
        for row in rows {
            assert!(!column.get(row));
            assert!(column.claim(row));
            assert!(column.get(row));
            assert!(rows.contains(&(row + 1)) || !column.get(row + 1));
        }
        assert_eq!(column.count(), rows.len());
        assert!(column.iter().eq(rows));
        let created = created(&column);
        assert!(created[0] && created[1] && created[7] && !created[2]);
        assert_eq!(created.iter().filter(|created| **created).count(), 3);
    }

    #[test]
    fn filters_a_mask() {
        let column = RuntimeColumn::new();
        let rows = [3, 70, FIRST * WORD + 1, FIRST * WORD * 3 + 9];
        rows.into_iter().for_each(|row| {
            column.set(row, true);
        });
        // a mask ending in the first, the second and beyond the third segment
        for len in [1, 2, FIRST, FIRST + 5, FIRST * 3 + 1, FIRST * 8] {
            for value in [true, false] {
                let mut mask = vec![u64::MAX; len];
                column.filter(&mut mask, value);
                for row in 0..len * WORD {
                    let kept = mask[row / WORD] >> (row % WORD) & 1 == 1;
                    assert_eq!(kept, rows.contains(&row) == value, "{row} {len} {value}");
                }
            }
        }
        // what is not in the mask stays out
        let mut mask = vec![0b1010u64; 2];
        column.filter(&mut mask, true);
        assert_eq!(mask, [0b1000, 0]);
        let mut mask = vec![0b1010u64; 2];
        column.filter(&mut mask, false);
        assert_eq!(mask, [0b0010, 0b1010]);
    }

    #[test]
    fn clones_share_rows() {
        let column = RuntimeColumn::new();
        let clone = column.clone();
        clone.set(3, true);
        clone.set(90_000, true);
        assert!(column.get(3) && column.get(90_000));
    }

    #[test]
    fn claimed_once() {
        let column = RuntimeColumn::new();
        let claims: usize = std::thread::scope(|scope| {
            let threads: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| usize::from(column.claim(500_000))))
                .collect();
            threads.into_iter().map(|t| t.join().unwrap()).sum()
        });
        assert_eq!(claims, 1);
        assert!(!column.claim(500_000));
        assert!(column.claim(6));
    }

    #[test]
    #[should_panic(expected = "at most u32::MAX rows")]
    fn set_out_of_range() {
        RuntimeColumn::new().set(RowSet::MAX_ROWS, true);
    }
}
