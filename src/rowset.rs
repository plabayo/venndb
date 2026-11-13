//! A set of rows, the building block of every column.

/// A sparse set turns dense once more than 1 in `DENSE_RATIO` rows is in it,
/// the point where a bitmask is at most twice the size of a list.
const DENSE_RATIO: usize = 64;

/// A sparse set of less rows than this is never turned dense.
const DENSE_MIN: usize = 16;

pub(crate) const WORD: usize = u64::BITS as usize;

/// A set of rows, identified by their index in the database.
///
/// Stored as a sorted list of rows as long as few rows are in it,
/// and as a bitmask with one bit per row once many are.
/// The two forms behave the same, the set picks the smallest one on its own.
#[derive(Debug, Clone, Default)]
pub struct RowSet {
    repr: Repr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Repr {
    /// The rows, sorted and unique.
    Sparse(Vec<u32>),
    /// One bit per row, without trailing zero words, and the number of ones.
    Dense(Vec<u64>, usize),
}

impl Default for Repr {
    fn default() -> Self {
        Self::Sparse(Vec::new())
    }
}

/// Sets are equal if they hold the same rows, whatever the form they are stored in.
impl PartialEq for RowSet {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl Eq for RowSet {}

/// The row as it is stored.
///
/// # Panics
///
/// Panics if the row is not below [`RowSet::MAX_ROWS`].
pub(crate) fn stored(row: usize) -> u32 {
    assert!(
        row < RowSet::MAX_ROWS,
        "venndb: a database holds at most u32::MAX rows"
    );
    row as u32
}

impl RowSet {
    /// The number of rows a database can hold: rows are numbered below it.
    pub const MAX_ROWS: usize = u32::MAX as usize;

    /// Create an empty set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            repr: Repr::Sparse(Vec::new()),
        }
    }

    /// The number of rows in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.repr {
            Repr::Sparse(rows) => rows.len(),
            Repr::Dense(_, ones) => *ones,
        }
    }

    /// Return `true` if no row is in the set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return `true` if the row is in the set.
    #[must_use]
    pub fn contains(&self, row: usize) -> bool {
        match &self.repr {
            Repr::Sparse(rows) => u32::try_from(row).is_ok_and(|r| rows.binary_search(&r).is_ok()),
            Repr::Dense(words, _) => words
                .get(row / WORD)
                .is_some_and(|word| word >> (row % WORD) & 1 == 1),
        }
    }

    /// Add a row to the set, returns `false` if it was already in it.
    ///
    /// Rows are cheapest to add in increasing order.
    ///
    /// # Panics
    ///
    /// Panics if the row is not below [`RowSet::MAX_ROWS`].
    pub fn insert(&mut self, row: usize) -> bool {
        let small = stored(row);
        match &mut self.repr {
            Repr::Sparse(rows) => {
                match rows.last() {
                    Some(last) if *last >= small => match rows.binary_search(&small) {
                        Ok(_) => return false,
                        Err(at) => rows.insert(at, small),
                    },
                    _ => rows.push(small),
                }
                let span = rows.last().map_or(0, |last| *last as usize + 1);
                if rows.len() >= DENSE_MIN && rows.len() as u64 * DENSE_RATIO as u64 > span as u64 {
                    self.repr = Repr::Dense(dense(rows, span), rows.len());
                }
                true
            }
            Repr::Dense(words, ones) => {
                let (at, bit) = (row / WORD, 1 << (row % WORD));
                if at >= words.len() {
                    // far beyond the last row: the set is thin again
                    if (*ones as u64 * DENSE_RATIO as u64 * 2) < row as u64 + 1 {
                        let mut rows = sparse(words, *ones + 1);
                        rows.push(small);
                        self.repr = Repr::Sparse(rows);
                        return true;
                    }
                    words.resize(at + 1, 0);
                }
                let added = words[at] & bit == 0;
                words[at] |= bit;
                *ones += usize::from(added);
                added
            }
        }
    }

    /// The `n`th row of the set in increasing order, counting from zero.
    #[must_use]
    pub fn nth(&self, n: usize) -> Option<usize> {
        match &self.repr {
            Repr::Sparse(rows) => rows.get(n).map(|row| *row as usize),
            Repr::Dense(words, _) => select(words, n),
        }
    }

    /// One more than the last row of the set, zero if empty.
    #[must_use]
    pub fn span(&self) -> usize {
        match &self.repr {
            Repr::Sparse(rows) => rows.last().map_or(0, |last| *last as usize + 1),
            // the last word is never zero, and counted apart to not overflow
            Repr::Dense(words, _) => words.last().map_or(0, |last| {
                (words.len() - 1) * WORD + (WORD - last.leading_zeros() as usize)
            }),
        }
    }

    /// Iterate over the rows in the set, in increasing order.
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        match &self.repr {
            Repr::Sparse(rows) => Iter(IterRepr::Sparse(rows.iter())),
            Repr::Dense(words, _) => Iter(IterRepr::Dense(Ones::new(words))),
        }
    }

    /// Release the memory the set does not need.
    pub fn shrink_to_fit(&mut self) {
        match &mut self.repr {
            Repr::Sparse(rows) => rows.shrink_to_fit(),
            Repr::Dense(words, _) => words.shrink_to_fit(),
        }
    }

    pub(crate) const fn repr(&self) -> &Repr {
        &self.repr
    }
}

impl FromIterator<usize> for RowSet {
    fn from_iter<I: IntoIterator<Item = usize>>(rows: I) -> Self {
        let mut set = Self::new();
        for row in rows {
            set.insert(row);
        }
        set
    }
}

impl<'a> IntoIterator for &'a RowSet {
    type Item = usize;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn dense(rows: &[u32], span: usize) -> Vec<u64> {
    let mut words = vec![0; span.div_ceil(WORD)];
    for row in rows {
        words[*row as usize / WORD] |= 1 << (*row as usize % WORD);
    }
    words
}

fn sparse(words: &[u64], capacity: usize) -> Vec<u32> {
    let mut rows = Vec::with_capacity(capacity);
    rows.extend(Ones::new(words).map(|row| row as u32));
    rows
}

/// An iterator over the rows of a [`RowSet`].
#[derive(Debug, Clone)]
pub struct Iter<'a>(IterRepr<'a>);

#[derive(Debug, Clone)]
enum IterRepr<'a> {
    Sparse(std::slice::Iter<'a, u32>),
    Dense(Ones<'a>),
}

impl Iterator for Iter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match &mut self.0 {
            IterRepr::Sparse(rows) => rows.next().map(|row| *row as usize),
            IterRepr::Dense(ones) => ones.next(),
        }
    }
}

/// An iterator over the indices of the bits set in a slice of words.
#[derive(Debug, Clone)]
pub(crate) struct Ones<'a> {
    words: &'a [u64],
    at: usize,
    word: u64,
}

impl<'a> Ones<'a> {
    pub(crate) fn new(words: &'a [u64]) -> Self {
        Self {
            words,
            at: 0,
            word: words.first().copied().unwrap_or(0),
        }
    }
}

impl Iterator for Ones<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        while self.word == 0 {
            self.at += 1;
            self.word = *self.words.get(self.at)?;
        }
        let bit = self.word.trailing_zeros() as usize;
        self.word &= self.word - 1;
        Some(self.at * WORD + bit)
    }
}

/// The index of the `n`th bit set in the words, counting from zero.
pub(crate) fn select(words: &[u64], mut n: usize) -> Option<usize> {
    for (at, word) in words.iter().enumerate() {
        let ones = word.count_ones() as usize;
        if n < ones {
            let mut word = *word;
            for _ in 0..n {
                word &= word - 1;
            }
            return Some(at * WORD + word.trailing_zeros() as usize);
        }
        n -= ones;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_dense(set: &RowSet) -> bool {
        matches!(set.repr, Repr::Dense(..))
    }

    #[test]
    fn empty() {
        let set = RowSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert!(!set.contains(0));
        assert_eq!(set.iter().next(), None);
        assert_eq!(set, RowSet::default());
    }

    #[test]
    fn insert_in_any_order() {
        let mut set = RowSet::new();
        assert!(set.insert(7));
        assert!(set.insert(3));
        assert!(set.insert(900));
        assert!(!set.insert(3));
        assert!(!set.insert(900));
        assert_eq!(set.len(), 3);
        assert_eq!(set.iter().collect::<Vec<_>>(), [3, 7, 900]);
        assert!(set.contains(7) && !set.contains(8) && !set.contains(usize::MAX));
    }

    #[test]
    fn few_rows_stay_sparse() {
        let set: RowSet = (0..DENSE_MIN - 1).collect();
        assert!(!is_dense(&set));
        let set: RowSet = (0..1000).map(|row| row * DENSE_RATIO * 2).collect();
        assert!(!is_dense(&set));
        assert_eq!(set.len(), 1000);
    }

    #[test]
    fn many_rows_turn_dense() {
        let mut set = RowSet::new();
        for row in 0..DENSE_MIN {
            assert!(set.insert(row * 2));
        }
        assert!(is_dense(&set));
        assert_eq!(set.len(), DENSE_MIN);
        assert!(!set.insert(4));
        assert!(set.insert(5));
        assert!(set.insert(1000));
        assert_eq!(set.len(), DENSE_MIN + 2);
        assert!(set.contains(5) && set.contains(1000) && !set.contains(999));
        assert!(!set.contains(100_000));
        let mut expected: Vec<usize> = (0..DENSE_MIN).map(|row| row * 2).collect();
        expected.extend([5, 1000]);
        expected.sort_unstable();
        assert!(set.iter().eq(expected));
    }

    #[test]
    fn dense_turns_sparse_when_thin() {
        let mut set: RowSet = (0..DENSE_MIN).collect();
        assert!(is_dense(&set));
        // not thin enough yet
        assert!(set.insert(DENSE_MIN * DENSE_RATIO * 2 - 1));
        assert!(is_dense(&set));
        assert!(set.insert(1_000_000));
        assert!(!is_dense(&set));
        assert_eq!(set.len(), DENSE_MIN + 2);
        assert_eq!(set.iter().last(), Some(1_000_000));
        assert!(set.contains(0) && set.contains(1_000_000));
    }

    #[test]
    fn same_rows_in_both_forms() {
        let rows: Vec<usize> = (0..5000).filter(|row| row % 3 != 1).collect();
        let dense: RowSet = rows.iter().copied().collect();
        assert!(is_dense(&dense));
        let sparse = RowSet {
            repr: Repr::Sparse(rows.iter().map(|row| *row as u32).collect()),
        };
        assert_eq!(dense.len(), sparse.len());
        assert!(dense.iter().eq(sparse.iter()));
        assert!((0..5100).all(|row| dense.contains(row) == sparse.contains(row)));
        assert!((&dense).into_iter().eq(rows.iter().copied()));
    }

    #[test]
    fn turns_dense_at_one_row_in_64() {
        // 16 rows over a span of 1024 is exactly 1 in 64: not dense yet
        let mut set: RowSet = (0..15).collect();
        assert!(set.insert(1023));
        assert!(!is_dense(&set));
        let mut set: RowSet = (0..15).collect();
        assert!(set.insert(1022));
        assert!(is_dense(&set));
        // too few rows, however close together
        let set: RowSet = (0..15).collect();
        assert!(!is_dense(&set));
    }

    #[test]
    fn turns_sparse_below_one_row_in_128() {
        // 16 rows and one more at a span of 2048 is exactly 1 in 128: still dense
        let mut set: RowSet = (0..16).collect();
        assert!(set.insert(2047));
        assert!(is_dense(&set));
        let mut set: RowSet = (0..16).collect();
        assert!(set.insert(2048));
        assert!(!is_dense(&set));
        assert_eq!(set.iter().last(), Some(2048));
        // a row within the words there are never makes it sparse
        let mut set: RowSet = (0..16).collect();
        assert!(set.insert(63));
        assert!(is_dense(&set));
    }

    #[test]
    fn shrinks() {
        let mut set = RowSet::new();
        set.insert(1);
        set.shrink_to_fit();
        assert!(matches!(&set.repr, Repr::Sparse(rows) if rows.capacity() == 1));
        let mut set: RowSet = (0..65).collect();
        set.shrink_to_fit();
        assert!(matches!(&set.repr, Repr::Dense(words, 65) if words.capacity() == 2));
    }

    #[test]
    fn nth_and_span() {
        assert_eq!(RowSet::new().span(), 0);
        assert_eq!(RowSet::new().nth(0), None);
        for step in [1, 3, 200] {
            let rows: Vec<usize> = (0..100).map(|row| row * step + 5).collect();
            let set: RowSet = rows.iter().copied().collect();
            assert_eq!(set.span(), 99 * step + 6);
            for (n, row) in rows.iter().enumerate() {
                assert_eq!(set.nth(n), Some(*row));
            }
            assert_eq!(set.nth(100), None);
        }
        let set: RowSet = (0..64).collect();
        assert_eq!(set.span(), 64);
    }

    #[test]
    fn shrink_keeps_rows() {
        for step in [1, 200] {
            let mut set: RowSet = (0..100).map(|row| row * step).collect();
            let rows: Vec<_> = set.iter().collect();
            set.shrink_to_fit();
            assert!(set.iter().eq(rows));
        }
    }

    #[test]
    #[should_panic(expected = "at most u32::MAX rows")]
    fn row_has_to_fit() {
        RowSet::new().insert(RowSet::MAX_ROWS);
    }

    #[test]
    fn last_row() {
        let last = RowSet::MAX_ROWS - 1;
        let mut set: RowSet = (0..DENSE_MIN).collect();
        assert!(set.insert(last));
        assert!(!set.insert(last));
        assert_eq!(set.span(), RowSet::MAX_ROWS);
        assert_eq!(set.nth(DENSE_MIN), Some(last));
        assert!(set.contains(last));
        assert_eq!(stored(last), u32::MAX - 1);
    }

    #[test]
    fn equal_whatever_the_form() {
        let dense: RowSet = (0..16).chain([1500]).collect();
        let sparse: RowSet = [1500].into_iter().chain(0..16).collect();
        assert!(is_dense(&dense) && !is_dense(&sparse));
        assert_eq!(dense, sparse);
        assert_eq!(sparse, dense);
        let other: RowSet = [1501].into_iter().chain(0..16).collect();
        assert_ne!(dense, other);
        assert_ne!(sparse, other);
        assert_ne!(sparse, (0..16).collect());
        assert_ne!(RowSet::new(), [0].into_iter().collect());
    }

    #[test]
    fn random_inserts_match_a_model() {
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let mut next = |limit: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % limit as u64) as usize
        };
        for round in 0..200 {
            let span = [70, 300, 5000, 200_000][round % 4];
            let mut set = RowSet::new();
            let mut model = std::collections::BTreeSet::new();
            for _ in 0..next(400) + 1 {
                let row = if next(4) == 0 {
                    next(span * 40)
                } else {
                    next(span)
                };
                assert_eq!(set.insert(row), model.insert(row));
                if let Repr::Dense(words, ones) = &set.repr {
                    assert_ne!(words.last(), Some(&0));
                    let counted: u32 = words.iter().map(|word| word.count_ones()).sum();
                    assert_eq!(counted as usize, *ones);
                }
            }
            assert_eq!(set.len(), model.len());
            assert!(set.iter().eq(model.iter().copied()));
            assert_eq!(set.span(), model.last().map_or(0, |last| last + 1));
            for (n, row) in model.iter().enumerate() {
                assert_eq!(set.nth(n), Some(*row));
                assert!(set.contains(*row));
                assert_eq!(set.contains(row + 1), model.contains(&(row + 1)));
            }
            assert_eq!(set.nth(model.len()), None);
        }
    }

    #[test]
    fn ones_and_select() {
        let words = [0b1010, 0, 1 << 63, 0b1];
        let ones: Vec<_> = Ones::new(&words).collect();
        assert_eq!(ones, [1, 3, 191, 192]);
        for (n, row) in ones.iter().enumerate() {
            assert_eq!(select(&words, n), Some(*row));
        }
        assert_eq!(select(&words, 4), None);
        assert_eq!(select(&[], 0), None);
        assert_eq!(Ones::new(&[]).next(), None);
        assert_eq!(Ones::new(&[0, 0]).next(), None);
        assert_eq!(select(&[u64::MAX], 63), Some(63));
    }
}
