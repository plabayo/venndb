//! A column of which the rows can be changed while the database is in use.

use crate::rowset::WORD;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// A set of rows that can be changed at any time, from any thread, without a lock.
///
/// Use it for state that changes while the database is queried,
/// such as rows that are disabled for a while. The column is not part
/// of the database: it has a fixed number of rows, and rows appended
/// to the database afterwards are never in it.
///
/// Cloning is cheap, clones share the same rows.
#[derive(Debug, Clone)]
pub struct RuntimeColumn {
    words: Arc<[AtomicU64]>,
    len: usize,
}

impl RuntimeColumn {
    /// Create a column for the given number of rows, none of them set.
    #[must_use]
    pub fn new(len: usize) -> Self {
        Self {
            words: (0..len.div_ceil(WORD)).map(|_| AtomicU64::new(0)).collect(),
            len,
        }
    }

    /// The number of rows the column is made for.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return `true` if the column is made for no rows at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Return `true` if the row is set. Rows out of range never are.
    #[must_use]
    pub fn get(&self, row: usize) -> bool {
        row < self.len && self.words[row / WORD].load(Ordering::Relaxed) >> (row % WORD) & 1 == 1
    }

    /// Set or unset a row, returns the value it had before.
    ///
    /// # Panics
    ///
    /// Panics if the row is out of range.
    pub fn set(&self, row: usize, value: bool) -> bool {
        assert!(row < self.len, "venndb: row {row} out of range");
        let bit = 1 << (row % WORD);
        let word = &self.words[row / WORD];
        let before = if value {
            word.fetch_or(bit, Ordering::Relaxed)
        } else {
            word.fetch_and(!bit, Ordering::Relaxed)
        };
        before & bit != 0
    }

    /// Set a row, returns `true` only for the one caller that did set it.
    ///
    /// # Panics
    ///
    /// Panics if the row is out of range.
    pub fn claim(&self, row: usize) -> bool {
        !self.set(row, true)
    }

    /// Unset all rows.
    pub fn clear(&self) {
        for word in self.words.iter() {
            word.store(0, Ordering::Relaxed);
        }
    }

    /// The number of rows set.
    #[must_use]
    pub fn count(&self) -> usize {
        self.words().map(|word| word.count_ones() as usize).sum()
    }

    /// Iterate over the rows set, in increasing order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.words().enumerate().flat_map(|(at, word)| {
            (0..WORD)
                .filter(move |bit| word >> bit & 1 == 1)
                .map(move |bit| at * WORD + bit)
        })
    }

    pub(crate) fn words(&self) -> impl ExactSizeIterator<Item = u64> + '_ {
        self.words.iter().map(|word| word.load(Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let column = RuntimeColumn::new(130);
        assert_eq!(column.len(), 130);
        assert!(!column.is_empty());
        assert_eq!(column.count(), 0);
        assert!(!column.set(0, true));
        assert!(!column.set(129, true));
        assert!(!column.set(64, true));
        assert!(column.set(64, true));
        assert!(column.get(0) && column.get(64) && column.get(129));
        assert!(!column.get(1) && !column.get(130) && !column.get(usize::MAX));
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
    fn empty() {
        let column = RuntimeColumn::new(0);
        assert!(column.is_empty());
        assert!(!column.get(0));
        assert_eq!(column.words().len(), 0);
        assert_eq!(RuntimeColumn::new(64).words().len(), 1);
        assert_eq!(RuntimeColumn::new(65).words().len(), 2);
    }

    #[test]
    fn clones_share_rows() {
        let column = RuntimeColumn::new(10);
        let clone = column.clone();
        clone.set(3, true);
        assert!(column.get(3));
    }

    #[test]
    fn claimed_once() {
        let column = RuntimeColumn::new(8);
        let claims: usize = std::thread::scope(|scope| {
            let threads: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| usize::from(column.claim(5))))
                .collect();
            threads.into_iter().map(|t| t.join().unwrap()).sum()
        });
        assert_eq!(claims, 1);
        assert!(!column.claim(5));
        assert!(column.claim(6));
    }

    #[test]
    #[should_panic(expected = "row 8 out of range")]
    fn set_out_of_range() {
        RuntimeColumn::new(8).set(8, true);
    }
}
