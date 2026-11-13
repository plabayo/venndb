//! Combining row sets into the rows that match a query.

use crate::rowset::{Ones, Repr, RowSet, WORD, select};
use crate::runtime::RuntimeColumn;

/// Start from the smallest set and test its rows one by one
/// as long as it holds less than 1 in `DRIVE_RATIO` rows,
/// combine whole bitmasks otherwise.
const DRIVE_RATIO: usize = 256;

/// The number of random rows tried before filtering all of them.
const RANDOM_TRIES: usize = 8;

/// A single condition a row has to meet.
#[derive(Debug, Clone)]
pub enum Filter<'a> {
    /// The row is in the set.
    Is(&'a RowSet),
    /// The row is not in the set.
    Not(&'a RowSet),
    /// The row is in at least one of the sets.
    AnyOf(Vec<&'a RowSet>),
    /// The row is set, or not set, in the runtime column.
    Runtime(&'a RuntimeColumn, bool),
}

impl Filter<'_> {
    fn contains(&self, row: usize) -> bool {
        match self {
            Self::Is(set) => set.contains(row),
            Self::Not(set) => !set.contains(row),
            Self::AnyOf(sets) => sets.iter().any(|set| set.contains(row)),
            Self::Runtime(column, value) => column.get(row) == *value,
        }
    }

    /// The work it takes to walk over all rows of the filter, if that is possible.
    fn walk_cost(&self) -> Option<usize> {
        let cost = |set: &RowSet| match set.repr() {
            Repr::Sparse(rows) => rows.len(),
            Repr::Dense(words, ones) => words.len() + ones,
        };
        match self {
            Self::Is(set) => Some(cost(set)),
            Self::AnyOf(sets) => Some(sets.iter().map(|set| cost(set)).sum()),
            Self::Not(_) | Self::Runtime(..) => None,
        }
    }
}

/// The filters of a query: a row matches if it meets all of them.
///
/// A plan has a fixed number of slots, each holding at most one filter,
/// for the filters that can be set more than once. Other filters are added
/// with [`Plan::and`].
#[derive(Debug, Clone)]
pub struct Plan<'a> {
    rows: usize,
    slots: Vec<Option<Filter<'a>>>,
    extra: Vec<Filter<'a>>,
}

impl<'a> Plan<'a> {
    /// Create a plan for a database with the given number of rows.
    #[must_use]
    pub fn new(rows: usize, slots: usize) -> Self {
        Self {
            rows,
            slots: vec![None; slots],
            extra: Vec::new(),
        }
    }

    /// Remove all filters.
    pub fn reset(&mut self) {
        self.slots.fill(None);
        self.extra.clear();
    }

    /// Set the filter of a slot, replacing the one it had.
    pub fn set(&mut self, slot: usize, filter: Filter<'a>) {
        self.slots[slot] = Some(filter);
    }

    /// Add a set to the [`Filter::AnyOf`] of a slot.
    ///
    /// Adding `None` adds no rows, but the slot does become a filter.
    pub fn or(&mut self, slot: usize, set: Option<&'a RowSet>) {
        if !matches!(self.slots[slot], Some(Filter::AnyOf(_))) {
            self.slots[slot] = Some(Filter::AnyOf(Vec::new()));
        }
        if let (Some(Filter::AnyOf(sets)), Some(set)) = (&mut self.slots[slot], set)
            && !sets.iter().any(|known| std::ptr::eq(*known, set))
        {
            sets.push(set);
        }
    }

    /// Add a filter next to the ones in the slots.
    pub fn and(&mut self, filter: Filter<'a>) {
        self.extra.push(filter);
    }

    /// The rows that meet all filters, `None` if there are no such rows.
    #[must_use]
    pub fn execute(&self) -> Option<Matches<'a>> {
        let rows = self.rows;
        let filters: Vec<&Filter<'a>> = self.slots.iter().flatten().chain(&self.extra).collect();
        if rows == 0 {
            return None;
        }
        let mut driver: Option<(usize, usize)> = None;
        for (at, filter) in filters.iter().enumerate() {
            if let Some(cost) = filter.walk_cost() {
                if cost == 0 {
                    return None;
                }
                if driver.is_none_or(|(best, _)| cost < best) {
                    driver = Some((cost, at));
                }
            }
        }
        match (filters.as_slice(), driver) {
            ([], _) => Some(Matches(Repr2::All(rows))),
            ([Filter::Is(set)], _) if set.span() <= rows => Some(Matches(Repr2::Set(set))),
            (_, Some((cost, at))) if cost.saturating_mul(DRIVE_RATIO) <= rows => {
                walk(rows, &filters, at)
            }
            _ => combine(rows, &filters),
        }
    }
}

/// Walk over the rows of one filter, keeping those that meet the others.
fn walk<'a>(rows: usize, filters: &[&Filter<'_>], driver: usize) -> Option<Matches<'a>> {
    let keep = |row: &usize| {
        *row < rows
            && filters
                .iter()
                .enumerate()
                .all(|(at, filter)| at == driver || filter.contains(*row))
    };
    let mut found: Vec<u32> = Vec::new();
    match filters[driver] {
        Filter::Is(set) => found.extend(set.iter().filter(keep).map(|row| row as u32)),
        Filter::AnyOf(sets) => {
            for set in sets {
                found.extend(set.iter().filter(keep).map(|row| row as u32));
            }
            if sets.len() > 1 {
                found.sort_unstable();
                found.dedup();
            }
        }
        Filter::Not(_) | Filter::Runtime(..) => unreachable!("not a filter to walk over"),
    }
    (!found.is_empty()).then_some(Matches(Repr2::Rows(found)))
}

/// Combine the filters as bitmasks of all rows.
fn combine<'a>(rows: usize, filters: &[&Filter<'_>]) -> Option<Matches<'a>> {
    let len = rows.div_ceil(WORD);
    let start = filters.iter().position(
        |filter| matches!(filter, Filter::Is(set) if matches!(set.repr(), Repr::Dense(..))),
    );
    let mut mask = match start.map(|at| filters[at]) {
        Some(Filter::Is(set)) => match set.repr() {
            Repr::Dense(words, _) => {
                let mut mask: Vec<u64> = words.iter().copied().take(len).collect();
                mask.resize(len, 0);
                mask
            }
            Repr::Sparse(_) => unreachable!("start is a dense set"),
        },
        _ => vec![u64::MAX; len],
    };
    let mut scratch = Vec::new();
    for (at, filter) in filters.iter().enumerate() {
        if Some(at) == start {
            continue;
        }
        match filter {
            Filter::Is(set) => match set.repr() {
                Repr::Dense(words, _) => and(&mut mask, words.iter().copied()),
                Repr::Sparse(found) => {
                    scratch.clear();
                    scratch.resize(len, 0);
                    for row in found.iter().map(|row| *row as usize) {
                        if let Some(word) = mask.get(row / WORD) {
                            scratch[row / WORD] |= word & (1 << (row % WORD));
                        }
                    }
                    std::mem::swap(&mut mask, &mut scratch);
                }
            },
            Filter::Not(set) => match set.repr() {
                Repr::Dense(words, _) => {
                    for (word, other) in mask.iter_mut().zip(words) {
                        *word &= !other;
                    }
                }
                Repr::Sparse(found) => {
                    for row in found.iter().map(|row| *row as usize) {
                        if let Some(word) = mask.get_mut(row / WORD) {
                            *word &= !(1 << (row % WORD));
                        }
                    }
                }
            },
            Filter::AnyOf(sets) => {
                scratch.clear();
                scratch.resize(len, 0);
                for set in sets {
                    match set.repr() {
                        Repr::Dense(words, _) => {
                            for (word, other) in scratch.iter_mut().zip(words) {
                                *word |= other;
                            }
                        }
                        Repr::Sparse(found) => {
                            for row in found.iter().map(|row| *row as usize) {
                                if let Some(word) = scratch.get_mut(row / WORD) {
                                    *word |= 1 << (row % WORD);
                                }
                            }
                        }
                    }
                }
                and(&mut mask, scratch.iter().copied());
            }
            Filter::Runtime(column, true) => and(&mut mask, column.words()),
            Filter::Runtime(column, false) => {
                for (word, other) in mask.iter_mut().zip(column.words()) {
                    *word &= !other;
                }
            }
        }
    }
    if !rows.is_multiple_of(WORD) {
        mask[len - 1] &= (1 << (rows % WORD)) - 1;
    }
    let ones: usize = mask.iter().map(|word| word.count_ones() as usize).sum();
    (ones > 0).then_some(Matches(Repr2::Mask(mask, ones)))
}

/// Keep the bits that are also set in the other words, which may be fewer.
fn and(mask: &mut [u64], other: impl ExactSizeIterator<Item = u64>) {
    let known = other.len().min(mask.len());
    for (word, other) in mask.iter_mut().zip(other) {
        *word &= other;
    }
    mask[known..].fill(0);
}

/// The rows that match a query, there is always at least one.
#[derive(Debug, Clone)]
pub struct Matches<'a>(Repr2<'a>);

#[derive(Debug, Clone)]
enum Repr2<'a> {
    /// All rows of a database with this many rows.
    All(usize),
    /// The rows of a set of the database itself.
    Set(&'a RowSet),
    /// The rows, sorted and unique.
    Rows(Vec<u32>),
    /// One bit per row, and the number of ones.
    Mask(Vec<u64>, usize),
}

impl Matches<'_> {
    /// The number of rows, never zero.
    #[expect(clippy::len_without_is_empty, reason = "matches are never empty")]
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.0 {
            Repr2::All(rows) => *rows,
            Repr2::Set(set) => set.len(),
            Repr2::Rows(rows) => rows.len(),
            Repr2::Mask(_, ones) => *ones,
        }
    }

    /// The `n`th row in increasing order, counting from zero.
    #[must_use]
    pub fn nth(&self, n: usize) -> Option<usize> {
        match &self.0 {
            Repr2::All(rows) => (n < *rows).then_some(n),
            Repr2::Set(set) => set.nth(n),
            Repr2::Rows(rows) => rows.get(n).map(|row| *row as usize),
            Repr2::Mask(words, _) => select(words, n),
        }
    }

    /// The first row.
    #[must_use]
    pub fn first(&self) -> usize {
        self.nth(0).expect("matches are never empty")
    }

    /// A random row.
    #[must_use]
    pub fn random(&self) -> usize {
        self.nth(crate::rng::below(self.len()))
            .expect("matches are never empty")
    }

    /// A random row for which the predicate returns `true`.
    ///
    /// Tries a few random rows first, to only test all rows if none of those do.
    #[must_use]
    pub fn random_where(&self, mut predicate: impl FnMut(usize) -> bool) -> Option<usize> {
        for _ in 0..RANDOM_TRIES.min(self.len()) {
            let row = self.random();
            if predicate(row) {
                return Some(row);
            }
        }
        self.filter(predicate).map(|matches| matches.random())
    }

    /// The rows for which the predicate returns `true`, `None` if there are none.
    #[must_use]
    pub fn filter(&self, mut predicate: impl FnMut(usize) -> bool) -> Option<Matches<'static>> {
        let rows: Vec<u32> = self
            .iter()
            .filter(|row| predicate(*row))
            .map(|row| row as u32)
            .collect();
        (!rows.is_empty()).then_some(Matches(Repr2::Rows(rows)))
    }

    /// Iterate over the rows, in increasing order.
    #[must_use]
    pub fn iter(&self) -> MatchesIter<'_> {
        MatchesIter(match &self.0 {
            Repr2::All(rows) => IterRepr::All(0..*rows),
            Repr2::Set(set) => IterRepr::Set(set.iter()),
            Repr2::Rows(rows) => IterRepr::Rows(rows.iter()),
            Repr2::Mask(words, _) => IterRepr::Mask(Ones::new(words)),
        })
    }
}

impl<'a> IntoIterator for &'a Matches<'_> {
    type Item = usize;
    type IntoIter = MatchesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the rows of [`Matches`].
#[derive(Debug, Clone)]
pub struct MatchesIter<'a>(IterRepr<'a>);

#[derive(Debug, Clone)]
enum IterRepr<'a> {
    All(std::ops::Range<usize>),
    Set(crate::rowset::Iter<'a>),
    Rows(std::slice::Iter<'a, u32>),
    Mask(Ones<'a>),
}

impl Iterator for MatchesIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match &mut self.0 {
            IterRepr::All(rows) => rows.next(),
            IterRepr::Set(rows) => rows.next(),
            IterRepr::Rows(rows) => rows.next().map(|row| *row as usize),
            IterRepr::Mask(ones) => ones.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Random(u64);

    impl Random {
        fn next(&mut self, limit: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % limit as u64) as usize
        }

        /// A set with about one in `one_in` of the rows.
        fn set(&mut self, rows: usize, one_in: usize) -> RowSet {
            (0..rows).filter(|_| self.next(one_in) == 0).collect()
        }
    }

    fn naive(rows: usize, filters: &[&Filter<'_>]) -> Vec<usize> {
        (0..rows)
            .filter(|row| filters.iter().all(|filter| filter.contains(*row)))
            .collect()
    }

    fn found(matches: Option<Matches<'_>>) -> Vec<usize> {
        let Some(matches) = matches else {
            return Vec::new();
        };
        let rows: Vec<usize> = matches.iter().collect();
        assert!(!rows.is_empty(), "matches are never empty");
        assert_eq!(matches.len(), rows.len());
        assert_eq!(matches.first(), rows[0]);
        for (n, row) in rows.iter().enumerate() {
            assert_eq!(matches.nth(n), Some(*row));
        }
        assert_eq!(matches.nth(rows.len()), None);
        assert!(rows.contains(&matches.random()));
        assert!((&matches).into_iter().eq(rows.iter().copied()));
        rows
    }

    #[test]
    fn both_ways_agree_with_a_scan() {
        let mut random = Random(0x1234_5678_9ABC_DEF1);
        for rows in [1, 63, 64, 65, 200, 1000, 5000] {
            let column = RuntimeColumn::new(rows / 2 + random.next(rows));
            for row in 0..column.len() {
                column.set(row, random.next(3) == 0);
            }
            let sets: Vec<RowSet> = [1, 2, 3, 10, 50, 100, 400, 1, 7, 300]
                .into_iter()
                .map(|one_in| {
                    let span = rows + random.next(2) * 70;
                    random.set(span, one_in)
                })
                .collect();
            for _ in 0..300 {
                let filters: Vec<Filter<'_>> = (0..1 + random.next(4))
                    .map(|_| {
                        let set = &sets[random.next(sets.len())];
                        match random.next(6) {
                            0 | 1 => Filter::Is(set),
                            2 => Filter::Not(set),
                            3 | 4 => Filter::AnyOf(
                                (0..random.next(4))
                                    .map(|_| &sets[random.next(sets.len())])
                                    .collect(),
                            ),
                            _ => Filter::Runtime(&column, random.next(2) == 0),
                        }
                    })
                    .collect();
                let filters: Vec<&Filter<'_>> = filters.iter().collect();
                let expected = naive(rows, &filters);
                assert_eq!(found(combine(rows, &filters)), expected, "{filters:?}");
                for (at, filter) in filters.iter().enumerate() {
                    if filter.walk_cost().is_some() {
                        assert_eq!(found(walk(rows, &filters, at)), expected, "{filters:?}");
                    }
                }
                let mut plan = Plan::new(rows, 0);
                filters
                    .iter()
                    .for_each(|filter| plan.and((*filter).clone()));
                assert_eq!(found(plan.execute()), expected, "{filters:?}");
            }
        }
    }

    #[test]
    fn no_filters_match_all_rows() {
        assert!(Plan::new(0, 0).execute().is_none());
        assert_eq!(found(Plan::new(3, 2).execute()), [0, 1, 2]);
    }

    #[test]
    fn a_single_set_is_borrowed() {
        let set: RowSet = [1, 5].into_iter().collect();
        let mut plan = Plan::new(6, 0);
        plan.and(Filter::Is(&set));
        let matches = plan.execute().unwrap();
        assert!(matches!(matches.0, Repr2::Set(_)));
        assert_eq!(found(Some(matches)), [1, 5]);
        // a set with rows the database does not have is not used as is
        let mut plan = Plan::new(5, 0);
        plan.and(Filter::Is(&set));
        assert_eq!(found(plan.execute()), [1]);
    }

    #[test]
    fn slots() {
        let (a, b, empty): (RowSet, RowSet, RowSet) = (
            [0, 1].into_iter().collect(),
            [1, 2].into_iter().collect(),
            RowSet::new(),
        );
        let mut plan = Plan::new(4, 2);
        plan.set(0, Filter::Is(&a));
        plan.set(0, Filter::Not(&a));
        assert_eq!(found(plan.execute()), [2, 3]);
        plan.or(0, Some(&a));
        plan.or(0, Some(&a));
        assert!(matches!(&plan.slots[0], Some(Filter::AnyOf(sets)) if sets.len() == 1));
        plan.or(0, Some(&b));
        plan.or(0, None);
        assert_eq!(found(plan.execute()), [0, 1, 2]);
        plan.or(1, Some(&b));
        assert_eq!(found(plan.execute()), [1, 2]);
        plan.and(Filter::Not(&a));
        assert_eq!(found(plan.execute()), [2]);
        plan.reset();
        assert_eq!(found(plan.execute()), [0, 1, 2, 3]);
        plan.or(1, None);
        assert_eq!(found(plan.execute()), []);
        plan.reset();
        plan.and(Filter::Is(&empty));
        plan.and(Filter::Not(&a));
        assert_eq!(found(plan.execute()), []);
    }

    #[test]
    fn small_sets_are_walked() {
        let rows = DRIVE_RATIO * 40;
        let small: RowSet = (0..40).map(|row| row * 200).collect();
        let big: RowSet = (0..rows).filter(|row| row % 2 == 0).collect();
        let mut plan = Plan::new(rows, 0);
        plan.and(Filter::Is(&big));
        plan.and(Filter::Is(&small));
        let matches = plan.execute().unwrap();
        assert!(matches!(matches.0, Repr2::Rows(_)));
        assert_eq!(matches.len(), 40);
        // one row too many to walk
        let small: RowSet = (0..41).map(|row| row * 200).collect();
        let mut plan = Plan::new(rows, 0);
        plan.and(Filter::Is(&big));
        plan.and(Filter::Is(&small));
        let matches = plan.execute().unwrap();
        assert!(matches!(matches.0, Repr2::Mask(..)));
        assert_eq!(matches.len(), 41);
    }

    #[test]
    fn filter_and_random_where() {
        let mut plan = Plan::new(100, 0);
        let odd: RowSet = (0..100).filter(|row| row % 2 == 1).collect();
        plan.and(Filter::Not(&odd));
        let matches = plan.execute().unwrap();
        assert_eq!(matches.len(), 50);
        let filtered = matches.filter(|row| row >= 90).unwrap();
        assert_eq!(found(Some(filtered)), [90, 92, 94, 96, 98]);
        assert!(matches.filter(|row| row > 98).is_none());

        for _ in 0..50 {
            assert_eq!(matches.random_where(|row| row == 42), Some(42));
            assert!(matches.random_where(|row| row < 10).unwrap() < 10);
        }
        assert_eq!(matches.random_where(|_| false), None);
        // a single row is not tested more often than needed
        let one = Plan::new(1, 0).execute().unwrap();
        let mut calls = 0;
        assert_eq!(
            one.random_where(|_| {
                calls += 1;
                false
            }),
            None
        );
        assert_eq!(calls, 2);
    }
}
